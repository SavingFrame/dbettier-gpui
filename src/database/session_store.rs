use std::collections::HashMap;

use gpui::{App, AppContext, Context, Entity, EventEmitter, Global, Subscription};

use super::{
    DatabaseConnectionProfile,
    session::{DatabaseSession, DatabaseSessionEvent, DatabaseSessionState},
};

#[derive(Clone)]
struct GlobalDatabaseSessionStore(Entity<DatabaseSessionStore>);

impl Global for GlobalDatabaseSessionStore {}

struct SessionEntry {
    session: Entity<DatabaseSession>,
    _observation: Subscription,
    _events: Subscription,
}

pub(crate) struct DatabaseSessionStore {
    sessions: HashMap<String, SessionEntry>,
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
        self.sessions.get(profile_id).map(|entry| &entry.session)
    }

    pub(crate) fn connect(
        &mut self,
        profile: DatabaseConnectionProfile,
        cx: &mut Context<Self>,
    ) -> Entity<DatabaseSession> {
        if let Some(session) = self.session(&profile.uuid)
            && matches!(
                session.read(cx).state(),
                DatabaseSessionState::Connecting | DatabaseSessionState::Connected { .. }
            )
        {
            return session.clone();
        }

        let profile_id = profile.uuid.clone();
        let session = cx.new(|cx| DatabaseSession::new(profile, cx));

        let observation = cx.observe(&session, |_, _, cx| cx.notify());
        let events = cx.subscribe(
            &session,
            |_: &mut DatabaseSessionStore, _, event: &DatabaseSessionEvent, cx| {
                cx.emit(event.clone());
            },
        );

        self.sessions.insert(
            profile_id,
            SessionEntry {
                session: session.clone(),
                _observation: observation,
                _events: events,
            },
        );
        cx.notify();
        session
    }

    pub(crate) fn disconnect(&mut self, profile_id: &str, cx: &mut Context<Self>) {
        let Some(entry) = self.sessions.remove(profile_id) else {
            return;
        };

        entry
            .session
            .update(cx, |session, cx| session.disconnect(cx));
        cx.notify();
    }
}
