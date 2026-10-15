use gpui::{App, AppContext as _, Context, Entity, FocusHandle, Subscription, Window};
use gpui_kit::component::{
    IndexPath,
    input::{InputEvent, InputState},
    select::{SelectEvent, SelectState},
};

use super::model::ConnectionInputMode;
use super::{ConnectionDetails, ConnectionDraft};
use crate::database::DatabaseConnectionProfile;

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
    // pub(super) status: ConnectionEditorStatus,
    // _subscriptions: Vec<Subscription>,
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
        let host = input("localhost", "Host or IP address", false, window, cx);
        let port = input("5432", "5432", false, window, cx);
        let database = input("", "Database name", false, window, cx);
        let username = input("", "Username", false, window, cx);
        let password = input("", "Password", true, window, cx);
        let initial_uri = profile
            .map(|profile| profile.generate_uri())
            .unwrap_or_default();
        let uri = input(
            &initial_uri,
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

        Self {
            focus_handle: cx.focus_handle(),
            profile_id: profile.map(|profile| profile.uuid.clone()),
            // Existing profiles contain a URI. Keep it intact instead of guessing at parsing rules.
            mode: if profile.is_some() {
                ConnectionInputMode::Uri
            } else {
                ConnectionInputMode::Fields
            },
            name,
            host,
            port,
            database,
            username,
            password,
            uri,
            ssl_mode,
            // status: ConnectionEditorStatus::Idle,
        }
    }

    pub(crate) fn draft(&self, cx: &App) -> ConnectionDraft {
        ConnectionDraft {
            profile_id: self.profile_id.clone(),
            name: self.name.read(cx).value().to_string(),
            details: match self.mode {
                ConnectionInputMode::Fields => ConnectionDetails::Fields {
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
                },
                ConnectionInputMode::Uri => {
                    ConnectionDetails::Uri(self.uri.read(cx).value().to_string())
                }
            },
        }
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
