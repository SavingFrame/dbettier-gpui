use std::collections::HashMap;

use gpui::{App, AppContext, Context, Entity, EventEmitter, Global};

use super::{
    DatabaseConnectionProfile,
    session::{DatabaseSession, DatabaseSessionEvent, DatabaseSessionState},
};

#[derive(Clone)]
struct GlobalDatabaseSessionStore(Entity<DatabaseSessionStore>);

impl Global for GlobalDatabaseSessionStore {}

pub(crate) struct DatabaseSessionStore {
    sessions: HashMap<String, Entity<DatabaseSession>>,
}

impl EventEmitter<DatabaseSessionEvent> for DatabaseSessionStore {}

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

    pub(crate) fn session(&self, profile_id: &str) -> Option<&Entity<DatabaseSession>> {
        self.sessions.get(profile_id)
    }

    pub(crate) fn connect(&mut self, profile: DatabaseConnectionProfile, cx: &mut Context<Self>) {
        if self.sessions.get(&profile.uuid).is_some_and(|session| {
            matches!(
                session.read(cx).state(),
                DatabaseSessionState::Connecting | DatabaseSessionState::Connected { .. }
            )
        }) {
            return;
        }

        let profile_id = profile.uuid.clone();
        let session = cx.new(|cx| DatabaseSession::new(profile, cx));

        cx.observe(&session, |_, _, cx| cx.notify()).detach();
        cx.subscribe(
            &session,
            |_: &mut DatabaseSessionStore, _, event: &DatabaseSessionEvent, cx| {
                cx.emit(event.clone());
            },
        )
        .detach();

        self.sessions.insert(profile_id, session);
        cx.notify();
    }

    pub(crate) fn load_tables(
        &mut self,
        profile_id: &str,
        schema_name: &str,
        cx: &mut Context<Self>,
    ) {
        let Some(session) = self.sessions.get(profile_id) else {
            return;
        };

        session.update(cx, |session, cx| session.load_tables(schema_name, cx));
    }

    pub(crate) fn disconnect(&mut self, profile_id: &str, cx: &mut Context<Self>) {
        let Some(session) = self.sessions.remove(profile_id) else {
            return;
        };

        session.update(cx, |session, cx| session.disconnect(cx));
        cx.notify();
    }
}
