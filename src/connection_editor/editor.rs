use gpui::{
    App, AppContext as _, Context, Entity, EventEmitter, FocusHandle, Subscription, Window,
};
use gpui_kit::component::{
    IndexPath,
    input::{InputEvent, InputState},
    select::{SelectEvent, SelectState},
};

use super::model::ConnectionInputMode;
use super::{ConnectionDetails, ConnectionDraft, ConnectionEditorEvent, ConnectionEditorStatus};
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
    pub(super) status: ConnectionEditorStatus,
    _subscriptions: Vec<Subscription>,
}

impl EventEmitter<ConnectionEditorEvent> for ConnectionEditor {}

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
        let uri = input(
            profile.map_or("", |profile| &profile.uri),
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
        let mut subscriptions = Vec::new();
        for state in [&name, &host, &port, &database, &username, &password, &uri] {
            subscriptions.push(cx.subscribe(state, |editor, _, event: &InputEvent, cx| {
                if matches!(event, InputEvent::Change) {
                    editor.clear_feedback(cx);
                }
            }));
        }
        subscriptions.push(cx.subscribe(
            &ssl_mode,
            |editor, _, _: &SelectEvent<Vec<&'static str>>, cx| {
                editor.clear_feedback(cx);
            },
        ));

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
            status: ConnectionEditorStatus::Idle,
            _subscriptions: subscriptions,
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

    pub(crate) fn set_status(&mut self, status: ConnectionEditorStatus, cx: &mut Context<Self>) {
        self.status = status;
        cx.notify();
    }

    pub(super) fn clear_feedback(&mut self, cx: &mut Context<Self>) {
        if !self.status.busy() {
            self.set_status(ConnectionEditorStatus::Idle, cx);
        }
    }

    pub(super) fn request_save(&mut self, cx: &mut Context<Self>) {
        if !self.status.busy() {
            self.set_status(
                ConnectionEditorStatus::Notice(
                    "Saving is not configured yet. No connection was saved.".to_owned(),
                ),
                cx,
            );
            cx.emit(ConnectionEditorEvent::SaveRequested(self.draft(cx)));
        }
    }

    pub(super) fn request_test(&mut self, cx: &mut Context<Self>) {
        if !self.status.busy() {
            self.set_status(
                ConnectionEditorStatus::Notice(
                    "Connection testing is not configured yet. No test was run.".to_owned(),
                ),
                cx,
            );
            cx.emit(ConnectionEditorEvent::TestRequested(self.draft(cx)));
        }
    }

    pub(super) fn cancel(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        cx.emit(ConnectionEditorEvent::Cancelled);
        window.remove_window();
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
