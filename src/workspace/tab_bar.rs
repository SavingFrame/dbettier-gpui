use gpui::{
    App, Context, Entity, Pixels, Window, base::ResizableState, component::tab::TabBar, div,
    prelude::*, px,
};

use crate::{
    database::{DatabaseConnectionProfile, session::DatabaseSession},
    workspace::{MIN_MAIN_CONTENT_WIDTH, query_editor::QueryEditor, table_view::TableView},
};
use gpui_kit::component::{
    ActiveTheme as _, WindowExt as _,
    notification::Notification,
    resizable::{h_resizable, resizable_panel, v_resizable},
};

pub(crate) struct WorkspaceTab {
    pub(crate) title: String,
    database_profile: Entity<DatabaseConnectionProfile>,
    database_session: Entity<DatabaseSession>,

    query_editor: Entity<QueryEditor>,
    table_view: Entity<TableView>,
    vertical_layout: Entity<ResizableState>,
}

const MIN_QUERY_EDITOR_HEIGHT: Pixels = px(140.);
const DEFAULT_QUERY_EDITOR_HEIGHT: Pixels = px(260.);
const MAX_QUERY_EDITOR_HEIGHT: Pixels = px(600.);
const MIN_TABLE_VIEW_HEIGHT: Pixels = px(120.);

impl Render for WorkspaceTab {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
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
                            .size_range(MIN_QUERY_EDITOR_HEIGHT..MAX_QUERY_EDITOR_HEIGHT)
                            .flex_none()
                            .overflow_hidden()
                            .child(self.query_editor.clone()),
                    ),
            )
    }
}
