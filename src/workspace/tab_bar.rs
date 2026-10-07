use gpui::{
    App, Context, Entity, Pixels, Window, base::ResizableState, component::tab::TabBar, div,
    prelude::*, px,
};

use crate::{
    database::{
        DatabaseConnectionProfile, profile_store::DatabaseProfileStore, session::DatabaseSession,
        session_store::DatabaseSessionStore,
    },
    workspace::{
        MIN_MAIN_CONTENT_WIDTH, TableTarget, query_editor::QueryEditor, table_view::TableView,
    },
};
use gpui_kit::component::{
    ActiveTheme as _, WindowExt as _,
    notification::Notification,
    resizable::{h_resizable, resizable_panel, v_resizable},
};

pub(crate) struct WorkspaceTab {
    pub(crate) title: String,
    database_session: Entity<DatabaseSession>,
    query_editor: Entity<QueryEditor>,
    table_view: Entity<TableView>,
    vertical_layout: Entity<ResizableState>,
}

const MIN_QUERY_EDITOR_HEIGHT: Pixels = px(140.);
const DEFAULT_QUERY_EDITOR_HEIGHT: Pixels = px(260.);
const MAX_QUERY_EDITOR_HEIGHT: Pixels = px(600.);
const MIN_TABLE_VIEW_HEIGHT: Pixels = px(120.);

impl WorkspaceTab {
    pub(crate) fn new(
        table: TableTarget,
        session: Entity<DatabaseSession>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Self {
        Self {
            title: table.table_name.clone(),
            database_session: session,
            query_editor: cx.new(|_| QueryEditor),
            table_view: cx.new(|cx| TableView::new(window, cx)),
            vertical_layout: cx.new(|_| ResizableState::default()),
        }
    }
}

impl Render for WorkspaceTab {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        div().child(
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
                        .size_range(MIN_QUERY_EDITOR_HEIGHT..MAX_QUERY_EDITOR_HEIGHT)
                        .flex_none()
                        .overflow_hidden()
                        .child(self.query_editor.clone()),
                ),
        )
    }
}
