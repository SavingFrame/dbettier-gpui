use gpui::{App, AppContext, Context, Entity, Global};

use super::{
    DatabaseConnectionProfile,
    profile_storage::{DatabaseStorage, FileDatabaseStorage},
};

#[derive(Clone)]
struct GlobalDatabaseProfileStore(Entity<DatabaseProfileStore>);

impl Global for GlobalDatabaseProfileStore {}

pub(crate) struct DatabaseProfileStore {
    profiles: Vec<DatabaseConnectionProfile>,
    load_error: Option<String>,
}

impl DatabaseProfileStore {
    pub(crate) fn new(
        database_storage: Result<FileDatabaseStorage, String>,
        cx: &mut Context<Self>,
    ) -> Self {
        let (database_storage, load_error) = match database_storage {
            Ok(database_storage) => (Some(database_storage), None),
            Err(error) => (None, Some(error)),
        };

        if let Some(database_storage) = database_storage {
            cx.spawn(async move |profile_store, cx| {
                let profiles = cx
                    .background_spawn(async move { database_storage.load() })
                    .await;

                if let Err(error) = profile_store.update(cx, |profile_store, cx| {
                    match profiles {
                        Ok(profiles) => {
                            profile_store.profiles = profiles;
                            profile_store.load_error = None;
                        }
                        Err(error) => {
                            profile_store.load_error =
                                Some(format!("failed to load database profiles: {error}"));
                        }
                    }
                    cx.notify();
                }) {
                    eprintln!("failed to update the global database store: {error}");
                }
            })
            .detach();
        }

        Self {
            profiles: Vec::new(),
            load_error,
        }
    }

    pub(crate) fn set_global(entity: Entity<Self>, cx: &mut App) {
        cx.set_global(GlobalDatabaseProfileStore(entity));
    }

    pub(crate) fn global(cx: &App) -> Entity<Self> {
        cx.global::<GlobalDatabaseProfileStore>().0.clone()
    }

    pub(crate) fn profiles(&self) -> &[DatabaseConnectionProfile] {
        &self.profiles
    }

    pub(crate) fn get(&self, profile_uuid: &str) -> Option<DatabaseConnectionProfile> {
        self.profiles()
            .iter()
            .find(|profile| profile.uuid == profile_uuid)
            .cloned()
    }

    pub(crate) fn load_error(&self) -> Option<&str> {
        self.load_error.as_deref()
    }
}
