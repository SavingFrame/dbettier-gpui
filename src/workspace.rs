mod database_tree;
mod query_editor;
mod status_bar;
mod table_view;

use gpui::{Context, Entity, Subscription, Window, div, prelude::*};

use self::{
    database_tree::DatabaseTree, query_editor::QueryEditor, status_bar::StatusBar,
    table_view::TableView,
};
use crate::{database::profile_store::DatabaseProfileStore, theme::AppTheme};

pub(crate) struct Workspace {
    profile_store: Entity<DatabaseProfileStore>,
    _profile_store_subscription: Subscription,
    database_tree: Entity<DatabaseTree>,
    table_view: Entity<TableView>,
    query_editor: Entity<QueryEditor>,
    status_bar: Entity<StatusBar>,
}

impl Workspace {
    pub(crate) fn new(cx: &mut Context<Self>) -> Self {
        let profile_store = DatabaseProfileStore::global(cx);
        let profile_store_subscription = cx.observe(&profile_store, |_, _, cx| cx.notify());

        Self {
            profile_store,
            _profile_store_subscription: profile_store_subscription,
            database_tree: cx.new(DatabaseTree::new),
            table_view: cx.new(|_| TableView),
            query_editor: cx.new(|_| QueryEditor),
            status_bar: cx.new(|_| StatusBar),
        }
    }
}

impl Render for Workspace {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let database_profile_load_error =
            self.profile_store.read(cx).load_error().map(str::to_owned);
        let theme = cx.global::<AppTheme>();

        div()
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
