mod database_tree;
mod query_editor;
mod status_bar;
mod table_view;

use gpui::{Context, Entity, Window, div, prelude::*};

use self::{
    database_tree::DatabaseTree, query_editor::QueryEditor, status_bar::StatusBar,
    table_view::TableView,
};
use crate::theme::AppTheme;

pub(crate) struct Workspace {
    database_tree: Entity<DatabaseTree>,
    table_view: Entity<TableView>,
    query_editor: Entity<QueryEditor>,
    status_bar: Entity<StatusBar>,
}

impl Workspace {
    pub(crate) fn new(cx: &mut Context<Self>) -> Self {
        Self {
            database_tree: cx.new(|_| DatabaseTree),
            table_view: cx.new(|_| TableView),
            query_editor: cx.new(|_| QueryEditor),
            status_bar: cx.new(|_| StatusBar),
        }
    }
}

impl Render for Workspace {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = cx.global::<AppTheme>();

        div()
            .size_full()
            .flex()
            .flex_col()
            .overflow_hidden()
            .bg(theme.background)
            .text_color(theme.text)
            .child(
                div()
                    .flex_1()
                    .min_h_0()
                    .flex()
                    .child(self.database_tree.clone())
                    .child(
                        div()
                            .flex_1()
                            .min_w_0()
                            .flex()
                            .flex_col()
                            .child(self.table_view.clone())
                            .child(self.query_editor.clone()),
                    ),
            )
            .child(self.status_bar.clone())
    }
}
