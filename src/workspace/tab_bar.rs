use gpui::{
    App, Context, Entity, EventEmitter, FocusHandle, Focusable, Window, div, prelude::*, px,
};
use gpui_kit::component::dock::{
    BasePanel, DockArea, DockLayout, DockPlacement, DockSkin, Panel, PanelEvent, panel_handle,
};

use super::{TableTarget, query_editor::QueryEditor, table_view::TableView};
use crate::{
    database::session::DatabaseSession,
    workspace::query::{QuerySource, QueryState, TableQuery},
};

pub(crate) struct WorkspaceTab {
    title: String,
    _database_session: Entity<DatabaseSession>,

    dock_area: Entity<DockArea>,
}

impl WorkspaceTab {
    pub(crate) fn new(
        table: TableTarget,
        session: Entity<DatabaseSession>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Self {
        // Each document owns its area so dragging a tool cannot detach it from its document.
        let (dock_area, _) =
            DockSkin::dock_area(format!("document-{}", cx.entity_id()), Some(1), window, cx);
        let table_query = TableQuery::new(table.clone());
        let query_state = cx.new(|_| QueryState::new(QuerySource::Table(table_query)));
        let query_editor =
            cx.new(|cx| QueryEditor::new(window, cx, query_state.clone(), session.clone()));
        let table_view = cx.new(|cx| TableView::new(window, cx, query_state.clone()));
        let center = DockLayout::tabs().panel_view(panel_handle(table_view), cx);
        let bottom = DockLayout::tabs().panel_view(panel_handle(query_editor), cx);
        dock_area.update(cx, |area, cx| {
            area.set_center(center, window, cx);
            area.set_dock(DockPlacement::Bottom, bottom, window, cx);
            area.set_dock_size(DockPlacement::Bottom, px(260.), window, cx);
            area.set_dock_collapsible(DockPlacement::Bottom, true, window, cx);
        });
        query_state.update(cx, |state, cx| state.execute(cx, session.clone()));
        Self {
            title: table.table_name,
            _database_session: session,
            dock_area,
        }
    }
}

impl EventEmitter<PanelEvent> for WorkspaceTab {}

impl Focusable for WorkspaceTab {
    fn focus_handle(&self, cx: &App) -> FocusHandle {
        self.dock_area.read(cx).focus_handle(cx)
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
            .child(self.dock_area.clone())
    }
}
