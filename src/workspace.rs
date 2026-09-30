mod database_tree;
mod query_editor;
mod status_bar;
mod table_view;

use gpui::{Context, Entity, Pixels, Subscription, Window, div, prelude::*, px};
use gpui_kit::component::{
    ActiveTheme as _, WindowExt as _,
    notification::Notification,
    resizable::{ResizableState, h_resizable, resizable_panel, v_resizable},
};

use self::{
    database_tree::DatabaseTree, query_editor::QueryEditor, status_bar::StatusBar,
    table_view::TableView,
};
use crate::database::{
    profile_store::DatabaseProfileStore, session::DatabaseSessionEvent,
    session_store::DatabaseSessionStore,
};

struct ConnectionErrorNotification;

const DEFAULT_DATABASE_TREE_WIDTH: Pixels = px(280.);
const MIN_DATABASE_TREE_WIDTH: Pixels = px(180.);
const MAX_DATABASE_TREE_WIDTH: Pixels = px(520.);
const MIN_MAIN_CONTENT_WIDTH: Pixels = px(320.);
const DEFAULT_QUERY_EDITOR_HEIGHT: Pixels = px(260.);
const MIN_QUERY_EDITOR_HEIGHT: Pixels = px(140.);
const MAX_QUERY_EDITOR_HEIGHT: Pixels = px(600.);
const MIN_TABLE_VIEW_HEIGHT: Pixels = px(120.);

pub(crate) struct Workspace {
    profile_store: Entity<DatabaseProfileStore>,
    _profile_store_subscription: Subscription,
    _session_store_subscription: Subscription,
    database_tree: Entity<DatabaseTree>,
    table_view: Entity<TableView>,
    query_editor: Entity<QueryEditor>,
    status_bar: Entity<StatusBar>,
    horizontal_layout: Entity<ResizableState>,
    vertical_layout: Entity<ResizableState>,
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

        Self {
            profile_store,
            _profile_store_subscription: profile_store_subscription,
            _session_store_subscription: session_store_subscription,
            database_tree: cx.new(DatabaseTree::new),
            table_view: cx.new(|cx| TableView::new(window, cx)),
            query_editor: cx.new(|_| QueryEditor),
            status_bar: cx.new(|_| StatusBar),
            horizontal_layout: cx.new(|_| ResizableState::default()),
            vertical_layout: cx.new(|_| ResizableState::default()),
        }
    }
}

impl Render for Workspace {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
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
                div().flex_1().min_h_0().min_w_0().child(
                    h_resizable("workspace-horizontal")
                        .with_state(&self.horizontal_layout)
                        .child(
                            resizable_panel()
                                .size(DEFAULT_DATABASE_TREE_WIDTH)
                                .size_range(MIN_DATABASE_TREE_WIDTH..MAX_DATABASE_TREE_WIDTH)
                                .flex_none()
                                .overflow_hidden()
                                .child(self.database_tree.clone()),
                        )
                        .child(
                            resizable_panel()
                                .size_range(MIN_MAIN_CONTENT_WIDTH..Pixels::MAX)
                                .overflow_hidden()
                                .child(
                                    v_resizable("workspace-vertical")
                                        .with_state(&self.vertical_layout)
                                        .child(
                                            resizable_panel()
                                                .size_range(MIN_TABLE_VIEW_HEIGHT..Pixels::MAX)
                                                .overflow_hidden()
                                                .child(self.table_view.clone()),
                                        )
                                        .child(
                                            resizable_panel()
                                                .size(DEFAULT_QUERY_EDITOR_HEIGHT)
                                                .size_range(
                                                    MIN_QUERY_EDITOR_HEIGHT
                                                        ..MAX_QUERY_EDITOR_HEIGHT,
                                                )
                                                .flex_none()
                                                .overflow_hidden()
                                                .child(self.query_editor.clone()),
                                        ),
                                ),
                        ),
                ),
            )
            .child(self.status_bar.clone())
    }
}
