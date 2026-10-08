use gpui::{App, Context, Entity, Subscription, Window, div, prelude::*, px};
use gpui_kit::assets::IconName;
use gpui_kit::component::{
    ActiveTheme as _, Disableable as _, Sizable as _, Size,
    button::{Button, ButtonVariants as _},
    dock::{BasePanel, Panel, PanelEvent},
    spinner::Spinner,
    table::{Column, DataTable, TableDelegate, TableState},
};

use crate::database::session::DatabaseSession;
use crate::workspace::query::{QueryState, QueryStatus};

struct ResultsTable {
    query_state: Entity<QueryState>,
}

impl ResultsTable {
    fn new(query_state: Entity<QueryState>) -> Self {
        Self { query_state }
    }
}

impl TableDelegate for ResultsTable {
    fn columns_count(&self, cx: &App) -> usize {
        self.query_state
            .read(cx)
            .results
            .as_ref()
            .map_or(0, |output| output.columns.len())
    }

    fn rows_count(&self, cx: &App) -> usize {
        self.query_state
            .read(cx)
            .results
            .as_ref()
            .map_or(0, |output| output.rows.len())
    }

    fn column(&self, column_index: usize, cx: &App) -> Column {
        self.query_state
            .read(cx)
            .results
            .as_ref()
            .and_then(|output| output.columns.get(column_index))
            .map(|column| Column::new(format!("column-{column_index}"), column.name.clone()))
            .unwrap_or_default()
    }

    fn loading(&self, cx: &App) -> bool {
        self.query_state.read(cx).is_loading()
    }

    fn render_loading(
        &mut self,
        _size: Size,
        _window: &mut Window,
        cx: &mut Context<TableState<Self>>,
    ) -> impl IntoElement {
        div()
            .size_full()
            .flex()
            .items_center()
            .justify_center()
            .gap_2()
            .text_color(cx.theme().muted_foreground)
            .child(Spinner::new())
            .child("Loading results…")
    }

    fn render_empty(
        &mut self,
        _window: &mut Window,
        cx: &mut Context<TableState<Self>>,
    ) -> impl IntoElement {
        let message = self
            .query_state
            .read(cx)
            .error_string
            .clone()
            .unwrap_or_else(|| "No rows returned".to_owned());

        div()
            .size_full()
            .flex()
            .items_center()
            .justify_center()
            .text_color(cx.theme().muted_foreground)
            .child(message)
    }

    fn render_td(
        &mut self,
        row_index: usize,
        column_index: usize,
        _window: &mut Window,
        cx: &mut Context<TableState<Self>>,
    ) -> impl IntoElement {
        let text = self
            .query_state
            .read(cx)
            .results
            .as_ref()
            .and_then(|output| output.rows.get(row_index))
            .and_then(|row| row.get(column_index))
            .map(ToString::to_string)
            .unwrap_or_else(|| "Missing cell".to_owned());
        div()
            .w_full()
            .min_w_0()
            .truncate()
            .text_color(cx.theme().foreground)
            .child(text)
    }
}

pub(crate) struct TableView {
    state: Entity<TableState<ResultsTable>>,
    query_state: Entity<QueryState>,
    session: Entity<DatabaseSession>,
    _query_observation: Subscription,
}

impl TableView {
    pub(crate) fn new(
        window: &mut Window,
        cx: &mut Context<Self>,
        query_state: Entity<QueryState>,
        session: Entity<DatabaseSession>,
    ) -> Self {
        let state = cx.new(|cx| {
            TableState::new(ResultsTable::new(query_state.clone()), window, cx)
                .row_selectable(true)
                .col_selectable(false)
        });
        let query_observation = cx.observe(&query_state, |view, _, cx| {
            view.state.update(cx, |state, cx| {
                state.refresh(cx);
                cx.notify();
            });
            cx.notify();
        });
        Self {
            state,
            query_state,
            session,
            _query_observation: query_observation,
        }
    }

    fn toolbar_button(
        id: &'static str,
        label: &'static str,
        icon: IconName,
        tooltip: &'static str,
    ) -> Button {
        Button::new(id)
            .ghost()
            .small()
            .icon(icon)
            .label(label)
            .tooltip(tooltip)
    }
}

impl gpui::Focusable for TableView {
    fn focus_handle(&self, cx: &gpui::App) -> gpui::FocusHandle {
        self.state.read(cx).focus_handle(cx)
    }
}

impl gpui::EventEmitter<PanelEvent> for TableView {}

impl BasePanel for TableView {
    fn panel_name(&self) -> &'static str {
        "QueryResults"
    }

    fn closable(&self, _: &gpui::App) -> bool {
        false
    }
}

impl Panel for TableView {
    fn title(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        "Results"
    }

    fn inner_padding(&self, _: &gpui::App) -> bool {
        false
    }
}

impl Render for TableView {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = cx.theme();
        let query_state = self.query_state.read(cx);
        let loading = query_state.is_loading();
        let disabled = loading || query_state.results.is_none();
        let row_count = self.state.read(cx).delegate().rows_count(cx);
        let summary = if loading {
            "Loading results…".to_owned()
        } else if let Some(error) = &query_state.error_string {
            error.clone()
        } else {
            format!("{row_count} rows")
        };

        div()
            .size_full()
            .min_w_0()
            .min_h_0()
            .flex()
            .flex_col()
            .overflow_hidden()
            .bg(theme.background)
            .child(
                div()
                    .h(px(40.))
                    .flex_none()
                    .flex()
                    .items_center()
                    .justify_between()
                    .px_3()
                    .border_b_1()
                    .border_color(theme.border)
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .gap_1()
                            .child(
                                Self::toolbar_button("add-row", "Row", IconName::Plus, "Add a row")
                                    .disabled(disabled),
                            )
                            .child(
                                Self::toolbar_button(
                                    "refresh-results",
                                    "Refresh",
                                    IconName::RefreshCw,
                                    "Refresh results",
                                )
                                .disabled(disabled)
                                .on_click(cx.listener(
                                    |view, _, _, cx| {
                                        let session = view.session.clone();
                                        view.query_state.update(cx, |query_state, cx| {
                                            if !matches!(query_state.status, QueryStatus::Running) {
                                                query_state.execute(cx, session);
                                            }
                                        })
                                    },
                                )),
                            )
                            .child(
                                Self::toolbar_button(
                                    "filter-results",
                                    "Filter",
                                    IconName::ListFilter,
                                    "Filter results",
                                )
                                .disabled(disabled),
                            )
                            .child(
                                Self::toolbar_button(
                                    "sort-results",
                                    "Sort",
                                    IconName::ArrowUpDown,
                                    "Sort results",
                                )
                                .disabled(disabled),
                            ),
                    )
                    .child(
                        div()
                            .text_sm()
                            .text_color(theme.muted_foreground)
                            .child(summary),
                    ),
            )
            .child(
                div()
                    .flex_1()
                    .min_w_0()
                    .min_h_0()
                    .overflow_hidden()
                    .child(DataTable::new(&self.state).bordered(false).small()),
            )
    }
}
