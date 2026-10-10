use gpui::{
    App, Context, Entity, EventEmitter, FocusHandle, Focusable, Window, div, prelude::*, px,
};
use gpui_kit::component::{
    dock::{BasePanel, Panel, PanelEvent},
    resizable::{ResizableState, resizable_panel, v_resizable},
};

use super::{query_editor::QueryEditor, table_view::TableView};
use crate::{
    database::session::DatabaseSession,
    workspace::{Target, query::QueryState},
};

pub(crate) struct WorkspaceTab {
    title: String,
    _database_session: Entity<DatabaseSession>,

    query_editor: Entity<QueryEditor>,
    table_view: Entity<TableView>,
    layout_state: Entity<ResizableState>,
}

impl WorkspaceTab {
    pub(crate) fn new(
        target: Target,
        session: Entity<DatabaseSession>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Self {
        let query_state = cx.new(|_| match target.clone() {
            Target::Table(table_target) => QueryState::from_table_target(table_target),
            Target::Sql(sql_target) => QueryState::from_sql_target("".to_string(), sql_target),
        });
        let title = match target {
            Target::Table(t) => t.table_name,
            Target::Sql(_) => "hello".to_string(),
        };
        let query_editor =
            cx.new(|cx| QueryEditor::new(window, cx, query_state.clone(), session.clone()));
        let table_view =
            cx.new(|cx| TableView::new(window, cx, query_state.clone(), session.clone()));
        let layout_state = cx.new(|_| ResizableState::default());
        query_state.update(cx, |state, cx| state.execute(cx, session.clone()));
        Self {
            title,
            _database_session: session,
            query_editor,
            table_view,
            layout_state,
        }
    }
}

impl EventEmitter<PanelEvent> for WorkspaceTab {}

impl Focusable for WorkspaceTab {
    fn focus_handle(&self, cx: &App) -> FocusHandle {
        self.query_editor.read(cx).focus_handle(cx)
    }
}

impl BasePanel for WorkspaceTab {
    fn panel_name(&self) -> &'static str {
        "WorkspaceDocument"
    }
}

impl Panel for WorkspaceTab {
    fn title(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        self.title.clone()
    }

    fn inner_padding(&self, _: &App) -> bool {
        false
    }
}

impl Render for WorkspaceTab {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        div()
            .size_full()
            .overflow_hidden()
            // Only the workspace dock handles document drops. Nested dock groups
            // would claim drops for documents they do not own.
            .child(
                v_resizable("document-layout")
                    .with_state(&self.layout_state)
                    .child(resizable_panel().child(self.table_view.clone()))
                    .child(
                        resizable_panel()
                            .size(px(260.))
                            .child(self.query_editor.clone()),
                    ),
            )
    }
}
