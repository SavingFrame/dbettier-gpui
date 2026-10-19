use gpui::{App, AppContext, Context, Entity, Global, Task};

use crate::database::{connection::ConnectionParameters, profile::DatabaseBackend, profile_store};

use super::{
    DatabaseConnectionProfile,
    profile_storage::{DatabaseStorage, FileDatabaseStorage},
};

#[derive(Clone)]
struct GlobalDatabaseProfileStore(Entity<DatabaseProfileStore>);

impl Global for GlobalDatabaseProfileStore {}

pub(crate) enum WriteStatus {
    Idle,
    Running(Task<()>),
    Failed(String),
}

pub(crate) struct DatabaseProfileStore {
    profiles: Vec<DatabaseConnectionProfile>,
    load_error: Option<String>,
    storage: Option<FileDatabaseStorage>,
    status: WriteStatus,
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
                let (profiles, database_storage) = cx
                    .background_spawn(async move { (database_storage.load(), database_storage) })
                    .await;

                if let Err(error) = profile_store.update(cx, |profile_store, cx| {
                    match profiles {
                        Ok(profiles) => {
                            profile_store.profiles = profiles;
                            profile_store.storage = Some(database_storage);
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
            storage: None,
            load_error,
            status: WriteStatus::Idle,
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

    pub(crate) fn save_existing(
        &mut self,
        profile_uuid: &str,
        name: String,
        connection_parameters: ConnectionParameters,
        cx: &mut Context<Self>,
    ) -> Result<(), String> {
        if matches!(self.status, WriteStatus::Running(_)) {
            return Err("Status shouldn't be running".to_string());
        }
        let storage = self
            .storage
            .clone()
            .ok_or_else(|| "Storage does not exists in the profile store".to_string())?;

        let mut profiles = self.profiles.clone();
        let profile = profiles
            .iter_mut()
            .find(|profile| profile.uuid == profile_uuid)
            .ok_or_else(|| format!("profile with {profile_uuid} not found"))?;
        profile.name = name;
        profile.fields = connection_parameters;
        let task = cx.spawn(async move |profile_store, cx| {
            let (save_result, profiles) = cx
                .background_spawn(async move {
                    let save_result = storage.save(&profiles);
                    (save_result, profiles)
                })
                .await;
            if let Err(error) = profile_store.update(cx, |profile_store, cx| {
                profile_store.status = match save_result {
                    Ok(()) => {
                        profile_store.profiles = profiles;
                        WriteStatus::Idle
                    }
                    Err(error) => WriteStatus::Failed(error.to_string()),
                };
                cx.notify();
            }) {
                eprintln!("Failed to update profile store: {error}")
            }
        });
        self.status = WriteStatus::Running(task);
        cx.notify();
        Ok(())
    }

    pub(crate) fn create(
        &mut self,
        name: String,
        connection_parameters: ConnectionParameters,
        backend: DatabaseBackend,
        cx: &mut Context<Self>,
    ) -> Result<(), String> {
        if matches!(self.status, WriteStatus::Running(_)) {
            return Err("Status shouldn't be running".to_string());
        }
        let storage = self
            .storage
            .clone()
            .ok_or_else(|| "Storage does not exists in the profile store".to_string())?;

        let mut profiles = self.profiles.clone();
        let profile = DatabaseConnectionProfile {
            name,
            uuid: uuid::Uuid::now_v7().to_string(),
            fields: connection_parameters,
            backend,
        };
        profiles.push(profile);
        let task = cx.spawn(async move |profile_store, cx| {
            let (save_result, profiles) = cx
                .background_spawn(async move {
                    let save_result = storage.save(&profiles);
                    (save_result, profiles)
                })
                .await;
            if let Err(error) = profile_store.update(cx, |profile_store, cx| {
                profile_store.status = match save_result {
                    Ok(()) => {
                        profile_store.profiles = profiles;
                        WriteStatus::Idle
                    }
                    Err(error) => WriteStatus::Failed(error.to_string()),
                };
                cx.notify();
            }) {
                eprintln!("Failed to update profile store: {error}")
            }
        });
        self.status = WriteStatus::Running(task);
        cx.notify();
        Ok(())
    }

    pub(crate) fn remove(
        &mut self,
        profile_uuid: &str,
        cx: &mut Context<Self>,
    ) -> Result<(), String> {
        if matches!(self.status, WriteStatus::Running(_)) {
            return Err("Status shouldn't be running".to_string());
        }
        let storage = self
            .storage
            .clone()
            .ok_or_else(|| "Storage does not exists in the profile store".to_string())?;

        let mut profiles = self.profiles.clone();
        let profile_index = profiles
            .iter()
            .position(|profile| profile.uuid == profile_uuid)
            .ok_or_else(|| format!("profile with {profile_uuid} not found"))?;
        profiles.remove(profile_index);
        let task = cx.spawn(async move |profile_store, cx| {
            let (save_result, profiles) = cx
                .background_spawn(async move {
                    let save_result = storage.save(&profiles);
                    (save_result, profiles)
                })
                .await;
            if let Err(error) = profile_store.update(cx, |profile_store, cx| {
                profile_store.status = match save_result {
                    Ok(()) => {
                        profile_store.profiles = profiles;
                        WriteStatus::Idle
                    }
                    Err(error) => WriteStatus::Failed(error.to_string()),
                };
                cx.notify();
            }) {
                eprintln!("Failed to update profile store: {error}")
            }
        });
        self.status = WriteStatus::Running(task);
        cx.notify();
        Ok(())
    }

    pub(crate) fn load_error(&self) -> Option<&str> {
        self.load_error.as_deref()
    }

    pub(crate) fn status(&self) -> &WriteStatus {
        &self.status
    }
}
