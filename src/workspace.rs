mod database_tree;
mod query_editor;
mod status_bar;
mod table_view;

use gpui::{
    Context, DragMoveEvent, Empty, Entity, Pixels, Subscription, Window, div, prelude::*, px,
};

use self::{
    database_tree::DatabaseTree, query_editor::QueryEditor, status_bar::StatusBar,
    table_view::TableView,
};
use crate::{
    database::{
        profile_store::DatabaseProfileStore, session::DatabaseSessionEvent,
        session_store::DatabaseSessionStore,
    },
    notifications::NotificationCenter,
    theme::AppTheme,
};

const DEFAULT_DATABASE_TREE_WIDTH: Pixels = px(280.);
const MIN_DATABASE_TREE_WIDTH: Pixels = px(180.);
const MAX_DATABASE_TREE_WIDTH: Pixels = px(520.);
const MIN_MAIN_CONTENT_WIDTH: Pixels = px(320.);
const DEFAULT_QUERY_EDITOR_HEIGHT: Pixels = px(260.);
const MIN_QUERY_EDITOR_HEIGHT: Pixels = px(140.);
const MAX_QUERY_EDITOR_HEIGHT: Pixels = px(600.);
const MIN_TABLE_VIEW_HEIGHT: Pixels = px(120.);
const RESIZE_HANDLE_SIZE: Pixels = px(8.);

#[derive(Clone)]
struct DraggedDatabaseTreeHandle;

#[derive(Clone)]
struct DraggedQueryEditorHandle;

pub(crate) struct Workspace {
    profile_store: Entity<DatabaseProfileStore>,
    _profile_store_subscription: Subscription,
    _session_store_subscription: Subscription,
    notification_center: Entity<NotificationCenter>,
    database_tree: Entity<DatabaseTree>,
    table_view: Entity<TableView>,
    query_editor: Entity<QueryEditor>,
    status_bar: Entity<StatusBar>,
    database_tree_width: Pixels,
    query_editor_height: Pixels,
}

impl Workspace {
    pub(crate) fn new(cx: &mut Context<Self>) -> Self {
        let profile_store = DatabaseProfileStore::global(cx);
        let session_store = DatabaseSessionStore::global(cx);
        let profile_store_subscription = cx.observe(&profile_store, |_, _, cx| cx.notify());
        let notification_center = cx.new(|_| NotificationCenter::new());
        let session_store_subscription = cx.subscribe(
            &session_store,
            |workspace, _, event: &DatabaseSessionEvent, cx| match event {
                DatabaseSessionEvent::ConnectionFailed {
                    profile_id,
                    profile_name,
                    error,
                } => {
                    workspace
                        .notification_center
                        .update(cx, |notification_center, cx| {
                            notification_center.show_error(
                                format!("database-connection-{profile_id}"),
                                format!("Could not connect to {profile_name}"),
                                error.clone(),
                                cx,
                            );
                        });
                }
            },
        );

        Self {
            profile_store,
            _profile_store_subscription: profile_store_subscription,
            _session_store_subscription: session_store_subscription,
            notification_center,
            database_tree: cx.new(DatabaseTree::new),
            table_view: cx.new(|_| TableView),
            query_editor: cx.new(|_| QueryEditor),
            status_bar: cx.new(|_| StatusBar),
            database_tree_width: DEFAULT_DATABASE_TREE_WIDTH,
            query_editor_height: DEFAULT_QUERY_EDITOR_HEIGHT,
        }
    }
}

impl Workspace {
    fn resize_database_tree(
        &mut self,
        event: &DragMoveEvent<DraggedDatabaseTreeHandle>,
        cx: &mut Context<Self>,
    ) {
        let maximum_width = (event.bounds.right() - event.bounds.left() - MIN_MAIN_CONTENT_WIDTH)
            .clamp(MIN_DATABASE_TREE_WIDTH, MAX_DATABASE_TREE_WIDTH);
        self.database_tree_width = (event.event.position.x - event.bounds.left())
            .clamp(MIN_DATABASE_TREE_WIDTH, maximum_width);
        cx.notify();
    }

    fn resize_query_editor(
        &mut self,
        event: &DragMoveEvent<DraggedQueryEditorHandle>,
        cx: &mut Context<Self>,
    ) {
        let maximum_height = (event.bounds.bottom() - event.bounds.top() - MIN_TABLE_VIEW_HEIGHT)
            .clamp(MIN_QUERY_EDITOR_HEIGHT, MAX_QUERY_EDITOR_HEIGHT);
        self.query_editor_height = (event.bounds.bottom() - event.event.position.y)
            .clamp(MIN_QUERY_EDITOR_HEIGHT, maximum_height);
        cx.notify();
    }
}

impl Render for Workspace {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let database_profile_load_error =
            self.profile_store.read(cx).load_error().map(str::to_owned);
        let theme = cx.global::<AppTheme>();

        div()
            .relative()
            .size_full()
            .flex()
            .flex_col()
            .overflow_hidden()
            .bg(theme.background)
            .text_color(theme.text)
            .when_some(database_profile_load_error, |workspace, error| {
                workspace.child(
                    div()
                        .flex_none()
                        .px_3()
                        .py_2()
                        .border_b_1()
                        .border_color(theme.border)
                        .bg(theme.panel_raised)
                        .text_sm()
                        .text_color(theme.text_subtle)
                        .child(error),
                )
            })
            .child(
                div()
                    .flex_1()
                    .min_h_0()
                    .flex()
                    .on_drag_move::<DraggedDatabaseTreeHandle>(cx.listener(
                        |workspace, event, _, cx| {
                            workspace.resize_database_tree(event, cx);
                        },
                    ))
                    .child(
                        div()
                            .w(self.database_tree_width)
                            .h_full()
                            .flex_none()
                            .overflow_hidden()
                            .child(self.database_tree.clone()),
                    )
                    .child(
                        div()
                            .relative()
                            .w(px(1.))
                            .h_full()
                            .flex_none()
                            .bg(theme.border)
                            .child(
                                div()
                                    .id("database-tree-resize-handle")
                                    .absolute()
                                    .left(-RESIZE_HANDLE_SIZE / 2.)
                                    .w(RESIZE_HANDLE_SIZE)
                                    .h_full()
                                    .cursor_col_resize()
                                    .block_mouse_except_scroll()
                                    .on_drag(DraggedDatabaseTreeHandle, |_, _, _, cx| {
                                        cx.new(|_| Empty)
                                    }),
                            ),
                    )
                    .child(
                        div()
                            .flex_1()
                            .min_w_0()
                            .flex()
                            .flex_col()
                            .on_drag_move::<DraggedQueryEditorHandle>(cx.listener(
                                |workspace, event, _, cx| {
                                    workspace.resize_query_editor(event, cx);
                                },
                            ))
                            .child(self.table_view.clone())
                            .child(
                                div()
                                    .relative()
                                    .w_full()
                                    .h(px(1.))
                                    .flex_none()
                                    .bg(theme.border)
                                    .child(
                                        div()
                                            .id("query-editor-resize-handle")
                                            .absolute()
                                            .top(-RESIZE_HANDLE_SIZE / 2.)
                                            .w_full()
                                            .h(RESIZE_HANDLE_SIZE)
                                            .cursor_row_resize()
                                            .block_mouse_except_scroll()
                                            .on_drag(DraggedQueryEditorHandle, |_, _, _, cx| {
                                                cx.new(|_| Empty)
                                            }),
                                    ),
                            )
                            .child(
                                div()
                                    .w_full()
                                    .h(self.query_editor_height)
                                    .flex_none()
                                    .overflow_hidden()
                                    .child(self.query_editor.clone()),
                            ),
                    ),
            )
            .child(self.status_bar.clone())
            .child(self.notification_center.clone())
    }
}
