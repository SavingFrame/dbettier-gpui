use gpui::{Context, Entity, Subscription, Window, div, prelude::*, px};
use gpui_kit::component::{
    ActiveTheme as _, WindowExt as _,
    dock::{DockArea, DockLayout, DockPlacement, DockSkin, PanelStyle, panel_handle},
    notification::Notification,
};

use super::status_bar::StatusBar;
use crate::{
    database::{
        profile_store::DatabaseProfileStore, session::DatabaseSessionEvent,
        session_store::DatabaseSessionStore, target::Target,
    },
    database_explorer::{DatabaseExplorer, DatabaseExplorerEvent},
    query_tab::QueryTab,
};

struct ConnectionErrorNotification;

pub(crate) struct Workspace {
    profile_store: Entity<DatabaseProfileStore>,
    _profile_store_subscription: Subscription,
    session_store: Entity<DatabaseSessionStore>,
    _session_store_subscription: Subscription,
    _database_explorer_subscription: Subscription,
    status_bar: Entity<StatusBar>,
    dock_area: Entity<DockArea>,
}

impl Workspace {
    pub(crate) fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
        let profile_store = DatabaseProfileStore::global(cx);
        let session_store = DatabaseSessionStore::global(cx);
        let profile_store_subscription = cx.observe(&profile_store, |_, _, cx| cx.notify());
        let session_store_subscription = cx.subscribe_in(
            &session_store,
            window,
            |_, _, event: &DatabaseSessionEvent, window, cx| match event {
                DatabaseSessionEvent::ConnectionFailed {
                    profile_id,
                    profile_name,
                    error,
                } => {
                    window.push_notification(
                        Notification::error(error.clone())
                            .id1::<ConnectionErrorNotification>(format!(
                                "database-connection-{profile_id}"
                            ))
                            .title(format!("Could not connect to {profile_name}")),
                        cx,
                    );
                }
            },
        );

        let database_explorer = cx.new(DatabaseExplorer::new);
        let database_explorer_subscription = cx.subscribe_in(
            &database_explorer,
            window,
            |workspace, _, event: &DatabaseExplorerEvent, window, cx| match event {
                DatabaseExplorerEvent::OpenTable(target) => {
                    workspace.open_table(target, window, cx)
                }
                DatabaseExplorerEvent::EditConnection(profile) => {
                    if let Err(error) = crate::connection_editor::open(profile.as_ref(), cx) {
                        window.push_notification(
                            Notification::error(format!(
                                "Could not open connection editor: {error}"
                            )),
                            cx,
                        );
                    }
                }
            },
        );
        let (dock_area, skin) = DockSkin::dock_area("workspace", Some(1), window, cx);
        skin.set_panel_style(PanelStyle::TabBar, cx);
        skin.set_close_button_visible(true, cx);
        let left = DockLayout::tabs().panel_view(panel_handle(database_explorer), cx);
        dock_area.update(cx, |area, cx| {
            area.set_center(DockLayout::tabs(), window, cx);
            area.set_dock(DockPlacement::Left, left, window, cx);
            area.set_dock_size(DockPlacement::Left, px(280.), window, cx);
            area.set_dock_collapsible(DockPlacement::Left, true, window, cx);
        });

        Self {
            profile_store,
            _profile_store_subscription: profile_store_subscription,
            session_store,
            _session_store_subscription: session_store_subscription,
            _database_explorer_subscription: database_explorer_subscription,
            status_bar: cx.new(|_| StatusBar),
            dock_area,
        }
    }

    fn open_table(&mut self, target: &Target, window: &mut Window, cx: &mut Context<Self>) {
        let Some(profile) = self.profile_store.read(cx).get(target.profile_id()) else {
            window.push_notification(
                Notification::error("The connection profile no longer exists"),
                cx,
            );
            return;
        };
        let session = self
            .session_store
            .update(cx, |store, cx| store.connect(profile, cx));
        let tab = cx.new(|cx| QueryTab::new(target.clone(), session, window, cx));
        let tab = panel_handle(tab);
        self.dock_area.update(cx, |area, cx| {
            area.add_panel_view(tab, DockPlacement::Center, None, window, cx);
        });
    }
}

impl Render for Workspace {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let database_profile_load_error =
            self.profile_store.read(cx).load_error().map(str::to_owned);
        let theme = cx.theme();

        div()
            .relative()
            .size_full()
            .flex()
            .flex_col()
            .overflow_hidden()
            .bg(theme.background)
            .text_color(theme.foreground)
            .when_some(database_profile_load_error, |workspace, error| {
                workspace.child(
                    div()
                        .flex_none()
                        .px_3()
                        .py_2()
                        .border_b_1()
                        .border_color(theme.border)
                        .bg(theme.muted)
                        .text_sm()
                        .text_color(theme.foreground)
                        .child(error),
                )
            })
            .child(
                div()
                    .flex_1()
                    .min_h_0()
                    .min_w_0()
                    .child(self.dock_area.clone()),
            )
            .child(self.status_bar.clone())
    }
}
