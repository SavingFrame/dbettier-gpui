use crate::runtime;
use gpui::{Context, EventEmitter};

use super::{
    catalog::{DatabaseSchema, LoadState},
    connection::DatabaseConnection,
    profile::DatabaseConnectionProfile,
};

pub(crate) struct DatabaseSession {
    state: DatabaseSessionState,
}

pub(crate) enum DatabaseSessionState {
    Connecting,
    Connected {
        connection: DatabaseConnection,
        schemas: LoadState<Vec<DatabaseSchema>>,
    },
    Failed(String),
}

#[derive(Clone)]
pub(crate) enum DatabaseSessionEvent {
    ConnectionFailed {
        profile_id: String,
        profile_name: String,
        error: String,
    },
}

impl EventEmitter<DatabaseSessionEvent> for DatabaseSession {}

impl DatabaseSession {
    pub(crate) fn new(profile: DatabaseConnectionProfile, cx: &mut Context<Self>) -> Self {
        let profile_id = profile.uuid.clone();
        let profile_name = profile.name.clone();

        cx.spawn(async move |session, cx| {
            let result = runtime::spawn(cx, async move {
                match DatabaseConnection::from_connection_profile(profile).await {
                    Ok(connection) => {
                        let schemas = connection.list_schemas().await;
                        Ok((connection, schemas))
                    }
                    Err(error) => Err(error),
                }
            })
            .await;

            if let Err(error) = session.update(cx, |session, cx| {
                session.state = match result {
                    Ok(Ok((connection, schemas))) => DatabaseSessionState::Connected {
                        connection,
                        schemas: match schemas {
                            Ok(schemas) => LoadState::Loaded(schemas),
                            Err(error) => LoadState::Failed(error.to_string()),
                        },
                    },
                    Ok(Err(error)) => DatabaseSessionState::Failed(error.to_string()),
                    Err(error) => DatabaseSessionState::Failed(format!(
                        "database connection task failed: {error}"
                    )),
                };

                if let DatabaseSessionState::Failed(error) = &session.state {
                    cx.emit(DatabaseSessionEvent::ConnectionFailed {
                        profile_id,
                        profile_name,
                        error: error.clone(),
                    });
                }

                cx.notify();
            }) {
                eprintln!("failed to update database session after connecting: {error}");
            }
        })
        .detach();

        Self {
            state: DatabaseSessionState::Connecting,
        }
    }

    pub(crate) fn state(&self) -> &DatabaseSessionState {
        &self.state
    }

    pub(crate) fn load_tables(&mut self, schema_name: &str, cx: &mut Context<Self>) {
        let DatabaseSessionState::Connected {
            connection,
            schemas: LoadState::Loaded(schemas),
        } = &mut self.state
        else {
            return;
        };
        let Some(schema) = schemas.iter_mut().find(|schema| schema.name == schema_name) else {
            return;
        };
        if matches!(schema.tables, LoadState::Loading | LoadState::Loaded(_)) {
            return;
        }

        schema.tables = LoadState::Loading;
        let connection = connection.clone();
        let schema_name = schema_name.to_owned();
        let schema_name_for_query = schema_name.clone();
        cx.notify();

        cx.spawn(async move |session, cx| {
            let result = runtime::spawn(cx, async move {
                connection.list_tables(&schema_name_for_query).await
            })
            .await;

            if let Err(error) = session.update(cx, |session, cx| {
                let DatabaseSessionState::Connected {
                    schemas: LoadState::Loaded(schemas),
                    ..
                } = &mut session.state
                else {
                    return;
                };
                let Some(schema) = schemas.iter_mut().find(|schema| schema.name == schema_name)
                else {
                    return;
                };

                schema.tables = match result {
                    Ok(Ok(tables)) => LoadState::Loaded(tables),
                    Ok(Err(error)) => LoadState::Failed(error.to_string()),
                    Err(error) => {
                        LoadState::Failed(format!("database table loading task failed: {error}"))
                    }
                };
                cx.notify();
            }) {
                eprintln!("failed to update database session after loading tables: {error}");
            }
        })
        .detach();
    }

    pub(crate) fn disconnect(&self, cx: &mut Context<Self>) {
        let DatabaseSessionState::Connected { connection, .. } = &self.state else {
            return;
        };

        let connection = connection.clone();
        runtime::spawn(cx, async move { connection.close().await }).detach();
    }
}
