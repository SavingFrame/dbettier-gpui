use gpui::{App, AppContext as _, Context, Entity, FocusHandle, Subscription, Window};
use gpui_kit::component::{IndexPath, input::InputState, select::SelectState};

use super::ConnectionDraft;
use super::model::ConnectionInputMode;
use crate::{
    connection_editor::model::{SaveConnectionStatus, TestConnectionStatus},
    database::{
        DatabaseConnectionProfile,
        connection::{ConnectionDetails, ConnectionParameters, DatabaseConnection},
        profile::DatabaseBackend,
        profile_store::{DatabaseProfileStore, WriteStatus},
    },
    runtime,
};

pub(crate) struct ConnectionEditor {
    pub(super) focus_handle: FocusHandle,
    pub(super) profile_id: Option<String>,
    pub(super) mode: ConnectionInputMode,
    pub(super) name: Entity<InputState>,
    pub(super) host: Entity<InputState>,
    pub(super) port: Entity<InputState>,
    pub(super) database: Entity<InputState>,
    pub(super) username: Entity<InputState>,
    pub(super) password: Entity<InputState>,
    pub(super) uri: Entity<InputState>,
    pub(super) ssl_mode: Entity<SelectState<Vec<&'static str>>>,
    pub(super) test_connection_status: TestConnectionStatus,
    _profile_store_observation: Subscription,
    pub(super) profile_store: Entity<DatabaseProfileStore>,
    pub(super) save_profile_status: SaveConnectionStatus,
}

impl ConnectionEditor {
    pub(crate) fn new(
        profile: Option<&DatabaseConnectionProfile>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Self {
        let name = input(
            profile.map_or("", |profile| &profile.name),
            "e.g. Local development",
            false,
            window,
            cx,
        );
        let host = input(
            &profile
                .map(|profile| profile.fields.host.clone())
                .unwrap_or("localhost".to_string()),
            "Host or IP address",
            false,
            window,
            cx,
        );
        let port = input(
            &profile
                .map(|profile| profile.fields.port.clone())
                .unwrap_or("5432".to_string()),
            "5432",
            false,
            window,
            cx,
        );
        let database = input(
            &profile
                .map(|profile| profile.fields.database.clone())
                .unwrap_or_default(),
            "Database name",
            false,
            window,
            cx,
        );
        let username = input(
            &profile
                .map(|profile| profile.fields.username.clone())
                .unwrap_or_default(),
            "Username",
            false,
            window,
            cx,
        );
        let password = input(
            &profile
                .map(|profile| profile.fields.password.clone())
                .unwrap_or_default(),
            "Password",
            true,
            window,
            cx,
        );
        let uri = input(
            &profile
                .map(|profile| profile.generate_uri())
                .unwrap_or_default(),
            "postgresql://user:password@host:5432/database",
            true,
            window,
            cx,
        );
        let ssl_mode = cx.new(|cx| {
            SelectState::new(
                vec![
                    "disable",
                    "allow",
                    "prefer",
                    "require",
                    "verify-ca",
                    "verify-full",
                ],
                Some(IndexPath::new(2)),
                window,
                cx,
            )
        });
        let profile_store = DatabaseProfileStore::global(cx);

        let profile_store_subscription =
            cx.observe_in(&profile_store, window, |editor, store, window, cx| {
                if matches!(editor.save_profile_status, SaveConnectionStatus::Running) {
                    match store.read(cx).status() {
                        WriteStatus::Idle => window.remove_window(),
                        WriteStatus::Failed(error) => {
                            editor.save_profile_status =
                                SaveConnectionStatus::Failed(error.to_string())
                        }
                        _ => (),
                    }
                }
                cx.notify();
            });

        Self {
            focus_handle: cx.focus_handle(),
            profile_id: profile.map(|profile| profile.uuid.clone()),
            // Existing profiles contain a URI. Keep it intact instead of guessing at parsing rules.
            mode: ConnectionInputMode::Fields,
            name,
            host,
            port,
            database,
            username,
            password,
            uri,
            ssl_mode,
            test_connection_status: TestConnectionStatus::Idle,
            profile_store: DatabaseProfileStore::global(cx),
            _profile_store_observation: profile_store_subscription,
            save_profile_status: SaveConnectionStatus::Idle,
        }
    }

    pub(crate) fn draft(&self, cx: &App) -> ConnectionDraft {
        ConnectionDraft {
            profile_id: self.profile_id.clone(),
            name: self.name.read(cx).value().to_string(),
            details: match self.mode {
                ConnectionInputMode::Fields => {
                    ConnectionDetails::Parameters(ConnectionParameters {
                        host: self.host.read(cx).value().to_string(),
                        port: self.port.read(cx).value().to_string(),
                        database: self.database.read(cx).value().to_string(),
                        username: self.username.read(cx).value().to_string(),
                        password: self.password.read(cx).value().to_string(),
                        ssl_mode: self
                            .ssl_mode
                            .read(cx)
                            .selected_value()
                            .copied()
                            .unwrap_or("prefer")
                            .to_owned(),
                    })
                }
                ConnectionInputMode::Uri => {
                    ConnectionDetails::Uri(self.uri.read(cx).value().to_string())
                }
            },
        }
    }

    pub(super) fn test_connection(
        &mut self,
        connection_details: ConnectionDetails,
        backend: DatabaseBackend,
        cx: &mut Context<Self>,
    ) {
        self.test_connection_status =
            TestConnectionStatus::Running(Some(cx.spawn(async move |editor, cx| {
                let result = runtime::spawn_result(cx, "connection test", async move {
                    DatabaseConnection::test_connection(connection_details, backend).await
                })
                .await;
                if let Err(error) = editor.update(cx, |editor, cx| {
                    editor.test_connection_status = match result {
                        Ok(message) => TestConnectionStatus::Finished(message),
                        Err(error) => TestConnectionStatus::Failed(error),
                    };
                    cx.notify();
                }) {
                    eprint!("failed to update connection test status: {error}")
                }
            })));
        cx.notify();
    }

    pub(super) fn save_profile(&mut self, cx: &mut Context<Self>) {
        if matches!(self.save_profile_status, SaveConnectionStatus::Running) {
            return;
        }
        let draft = self.draft(cx);
        let ConnectionDetails::Parameters(parameters) = draft.details else {
            eprint!("We do not support URI parameters yet");
            return;
        };
        let result = self
            .profile_store
            .update(cx, |profile_store, cx| match draft.profile_id {
                Some(profile_id) => {
                    profile_store.save_existing(&profile_id, draft.name, parameters, cx)
                }
                None => {
                    profile_store.create(draft.name, parameters, DatabaseBackend::PostgreSql, cx)
                }
            });
        match result {
            Ok(()) => self.save_profile_status = SaveConnectionStatus::Running,
            Err(error) => self.save_profile_status = SaveConnectionStatus::Failed(error),
        }
        cx.notify();
    }
}

fn input(
    value: &str,
    placeholder: &'static str,
    secret: bool,
    window: &mut Window,
    cx: &mut Context<ConnectionEditor>,
) -> Entity<InputState> {
    let value = value.to_owned();
    cx.new(|cx| {
        InputState::new(window, cx)
            .default_value(value)
            .placeholder(placeholder)
            .masked(secret)
    })
}
