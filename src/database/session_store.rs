use std::collections::HashMap;

use gpui::{App, AppContext, Context, Entity, Global};
use uuid::Uuid;

use super::{DatabaseConnection, DatabaseConnectionProfile};

#[derive(Clone)]
struct GlobalDatabaseSessionStore(Entity<DatabaseSessionStore>);

impl Global for GlobalDatabaseSessionStore {}

pub(crate) struct DatabaseSessionStore {
    sessions: HashMap<String, DatabaseSession>,
}

pub(crate) struct DatabaseSession {
    connection_attempt_id: Uuid,
    state: DatabaseSessionState,
}

pub(crate) enum DatabaseSessionState {
    Connecting,
    Connected(DatabaseConnection),
    Failed(String),
}

impl DatabaseSessionStore {
    pub(crate) fn new() -> Self {
        Self {
            sessions: HashMap::new(),
        }
    }

    pub(crate) fn set_global(entity: Entity<Self>, cx: &mut App) {
        cx.set_global(GlobalDatabaseSessionStore(entity));
    }

    pub(crate) fn global(cx: &App) -> Entity<Self> {
        cx.global::<GlobalDatabaseSessionStore>().0.clone()
    }

    pub(crate) fn session(&self, profile_id: &str) -> Option<&DatabaseSession> {
        self.sessions.get(profile_id)
    }

    pub(crate) fn connect(&mut self, profile: DatabaseConnectionProfile, cx: &mut Context<Self>) {
        if self.sessions.get(&profile.uuid).is_some_and(|session| {
            matches!(
                session.state,
                DatabaseSessionState::Connecting | DatabaseSessionState::Connected(_)
            )
        }) {
            return;
        }

        let profile_id = profile.uuid.clone();
        let connection_attempt_id = Uuid::new_v4();

        self.sessions.insert(
            profile_id.clone(),
            DatabaseSession {
                connection_attempt_id,
                state: DatabaseSessionState::Connecting,
            },
        );
        cx.notify();

        cx.spawn(async move |session_store, cx| {
            let result = cx
                .background_spawn(DatabaseConnection::from_connection_profile(profile))
                .await;

            if let Err(error) = session_store.update(cx, |session_store, cx| {
                let Some(session) = session_store.sessions.get_mut(&profile_id) else {
                    return;
                };

                if session.connection_attempt_id != connection_attempt_id {
                    return;
                }

                session.state = match result {
                    Ok(connection) => DatabaseSessionState::Connected(connection),
                    Err(error) => DatabaseSessionState::Failed(error.to_string()),
                };
                cx.notify();
            }) {
                eprintln!("failed to update the global database session store: {error}");
            }
        })
        .detach();
    }

    pub(crate) fn disconnect(&mut self, profile_id: &str, cx: &mut Context<Self>) {
        let Some(session) = self.sessions.remove(profile_id) else {
            return;
        };

        cx.notify();

        if let DatabaseSessionState::Connected(connection) = session.state {
            cx.background_spawn(async move { connection.close().await })
                .detach();
        }
    }
}

impl DatabaseSession {
    pub(crate) fn state(&self) -> &DatabaseSessionState {
        &self.state
    }
}
