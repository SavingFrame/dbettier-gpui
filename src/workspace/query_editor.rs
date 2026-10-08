use gpui::{
    App, Context, Entity, FocusHandle, Focusable, Subscription, Window, div, prelude::*, px,
};

use gpui_kit::{
    assets::IconName,
    component::{
        ActiveTheme as _, Disableable as _, Sizable as _,
        button::{Button, ButtonVariants as _},
        dock::{BasePanel, Panel, PanelEvent},
        input::{Editor, EditorState, TabSize},
    },
};

use crate::{
    database::session::DatabaseSession,
    workspace::query::{QueryState, QueryStatus},
};

pub(crate) struct QueryEditor {
    editor: Entity<EditorState>,
    query_state: Entity<QueryState>,
    database_session: Entity<DatabaseSession>,
    _query_observation: Subscription,
}

impl QueryEditor {
    pub(crate) fn new(
        window: &mut Window,
        cx: &mut Context<Self>,
        query_state: Entity<QueryState>,
        database_session: Entity<DatabaseSession>,
    ) -> Self {
        let query_observation =
            cx.observe_in(&query_state, window, |view, query_state, window, cx| {
                let sql = query_state.read(cx).query.sql();
                view.editor.update(cx, |editor, cx| {
                    editor.set_value(sql, window, cx);
                });
                cx.notify();
            });
        let editor = cx.new(|cx| {
            EditorState::new(window, cx)
                .language("sql")
                .line_number(true)
                .folding(true)
                .tab_size(TabSize {
                    tab_size: 4,
                    hard_tabs: false,
                })
                .searchable(true)
                .default_value(query_state.read(cx).query.sql())
        });
        Self {
            editor,
            query_state,
            database_session,
            _query_observation: query_observation,
        }
    }
}

impl Focusable for QueryEditor {
    fn focus_handle(&self, cx: &App) -> FocusHandle {
        self.editor.read(cx).focus_handle(cx)
    }
}

impl gpui::EventEmitter<PanelEvent> for QueryEditor {}

impl BasePanel for QueryEditor {
    fn panel_name(&self) -> &'static str {
        "QueryEditor"
    }

    fn closable(&self, _: &gpui::App) -> bool {
        false
    }
}

impl Panel for QueryEditor {
    fn title(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        "Query editor"
    }

    fn inner_padding(&self, _: &gpui::App) -> bool {
        false
    }
}

impl Render for QueryEditor {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let running = matches!(self.query_state.read(cx).status, QueryStatus::Running);
        div()
            .size_full()
            .flex()
            .flex_col()
            .min_h_0()
            .overflow_hidden()
            .bg(cx.theme().background)
            .child(
                div()
                    .h(px(40.))
                    .flex_none()
                    .flex()
                    .items_center()
                    .px_3()
                    .border_b_1()
                    .border_color(cx.theme().border)
                    .child(
                        Button::new("run-query")
                            .ghost()
                            .small()
                            .icon(IconName::Play)
                            .label("Run")
                            .disabled(running)
                            .on_click(cx.listener(|view, _, _, cx| {
                                let session = view.database_session.clone();
                                view.query_state.update(cx, |query_state, cx| {
                                    if !matches!(query_state.status, QueryStatus::Running) {
                                        query_state.execute(cx, session);
                                    }
                                })
                            })),
                    ),
            )
            .child(
                div()
                    .flex_1()
                    .min_h_0()
                    .overflow_hidden()
                    .child(Editor::new(&self.editor).h_full().bordered(false)),
            )
    }
}
