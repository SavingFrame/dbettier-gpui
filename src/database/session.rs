use std::collections::HashMap;

use crate::{database::result::QueryOutput, runtime};
use gpui::{Context, EventEmitter, Task};

use super::{
    catalog::{DatabaseSchema, LoadState},
    connection::DatabaseConnection,
    profile::DatabaseConnectionProfile,
};

pub(crate) struct DatabaseSession {
    state: DatabaseSessionState,
    connection_task: Option<Task<()>>,
    table_tasks: HashMap<String, Task<()>>,
}

pub(crate) enum DatabaseSessionState {
    Disconnected,
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
        let connection_task = Self::connect(profile, cx);

        Self {
            state: DatabaseSessionState::Connecting,
            connection_task: Some(connection_task),
            table_tasks: HashMap::new(),
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
        let task_schema_name = schema_name.clone();
        cx.notify();

        let task = cx.spawn(async move |session, cx| {
            let schema_name_for_query = task_schema_name.clone();
            let result = runtime::spawn_result(cx, "database table loading", async move {
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
                let Some(schema) = schemas
                    .iter_mut()
                    .find(|schema| schema.name == task_schema_name)
                else {
                    return;
                };

                schema.tables = LoadState::from_result(result);
                cx.notify();
            }) {
                eprintln!("failed to update database session after loading tables: {error}");
            }
        });
        self.table_tasks.insert(schema_name, task);
    }

    pub(crate) fn connect(profile: DatabaseConnectionProfile, cx: &mut Context<Self>) -> Task<()> {
        let profile_id = profile.uuid.clone();
        let profile_name = profile.name.clone();
        cx.spawn(async move |session, cx| {
            let result = runtime::spawn_result(cx, "database connection", async move {
                let connection = DatabaseConnection::from_connection_profile(profile).await?;
                let schemas = connection.list_schemas().await;
                Ok::<_, super::connection::DatabaseError>((connection, schemas))
            })
            .await;

            if let Err(error) = session.update(cx, |session, cx| {
                session.state = match result {
                    Ok((connection, schemas)) => DatabaseSessionState::Connected {
                        connection,
                        schemas: LoadState::from_result(schemas),
                    },
                    Err(error) => {
                        cx.emit(DatabaseSessionEvent::ConnectionFailed {
                            profile_id,
                            profile_name,
                            error: error.clone(),
                        });
                        DatabaseSessionState::Failed(error)
                    }
                };
                cx.notify();
            }) {
                eprintln!("failed to update database session after connecting: {error}");
            }
        })
    }

    pub(crate) fn disconnect(&mut self, cx: &mut Context<Self>) {
        self.connection_task = None;
        self.table_tasks.clear();
        let state = std::mem::replace(&mut self.state, DatabaseSessionState::Disconnected);
        cx.notify();

        if let DatabaseSessionState::Connected { connection, .. } = state {
            // Pool shutdown must finish even if the registry releases this session.
            cx.spawn(async move |_, cx| {
                if let Err(error) =
                    runtime::spawn(cx, async move { connection.close().await }).await
                {
                    eprintln!("database disconnection task failed: {error}");
                }
            })
            .detach();
        }
    }

    pub(crate) fn reconnect(&mut self, cx: &mut Context<Self>, profile: DatabaseConnectionProfile) {
        self.disconnect(cx);
        let task = Self::connect(profile, cx);
        self.connection_task = Some(task);
        self.state = DatabaseSessionState::Connecting;
        cx.notify();
    }

    pub(crate) fn execute_query(
        &self,
        cx: &mut Context<Self>,
        query: String,
    ) -> Result<Task<Result<QueryOutput, String>>, String> {
        let DatabaseSessionState::Connected { connection, .. } = &self.state else {
            return Err("Cannot execute query: database is not ocnnected".to_owned());
        };
        let connection = connection.clone();
        Ok(runtime::spawn_result(cx, "query execution", async move {
            connection.execute_query(query).await
        }))
    }
}
