use gpui::{App, Context, Entity, Window, div, prelude::*, px};
use gpui_kit::assets::IconName;
use gpui_kit::component::{
    ActiveTheme as _, Sizable as _,
    button::{Button, ButtonVariants as _},
    table::{Column, DataTable, TableDelegate, TableState},
};

const COLUMNS: [(&str, f32); 6] = [
    ("#", 56.),
    ("id", 110.),
    ("name", 260.),
    ("category", 170.),
    ("price", 120.),
    ("updated_at", 150.),
];

// Placeholder results until query execution supplies a result set.
const SAMPLE_ROWS: [[&str; 6]; 9] = [
    [
        "1",
        "1042",
        "Wireless Keyboard",
        "Accessories",
        "$89.00",
        "2025-04-18",
    ],
    [
        "2",
        "1043",
        "Studio Display Stand",
        "Displays",
        "$149.00",
        "2025-04-18",
    ],
    [
        "3",
        "1044",
        "USB-C Dock",
        "Accessories",
        "$119.00",
        "2025-04-17",
    ],
    [
        "4",
        "1045",
        "Mechanical Keyboard",
        "Accessories",
        "$129.00",
        "2025-04-17",
    ],
    [
        "5",
        "1046",
        "Ergonomic Mouse",
        "Accessories",
        "$79.00",
        "2025-04-16",
    ],
    [
        "6",
        "1047",
        "27-inch Monitor",
        "Displays",
        "$449.00",
        "2025-04-15",
    ],
    [
        "7",
        "1048",
        "Laptop Stand",
        "Office",
        "$64.00",
        "2025-04-15",
    ],
    ["8", "1049", "Desk Mat", "Office", "$35.00", "2025-04-14"],
    ["9", "1050", "Webcam Light", "Video", "$54.00", "2025-04-14"],
];

struct ResultsTable {
    columns: Vec<Column>,
    rows: Vec<Vec<String>>,
}

impl ResultsTable {
    fn sample() -> Self {
        Self {
            columns: COLUMNS
                .into_iter()
                .map(|(name, width)| Column::new(name, name).width(px(width)).movable(false))
                .collect(),
            rows: SAMPLE_ROWS
                .into_iter()
                .map(|row| row.into_iter().map(str::to_owned).collect())
                .collect(),
        }
    }

    fn cell(&self, row: usize, column: usize) -> &str {
        self.rows
            .get(row)
            .and_then(|row| row.get(column))
            .map(String::as_str)
            .unwrap_or_default()
    }
}

impl TableDelegate for ResultsTable {
    fn columns_count(&self, _cx: &App) -> usize {
        self.columns.len()
    }

    fn rows_count(&self, _cx: &App) -> usize {
        self.rows.len()
    }

    fn column(&self, column: usize, _cx: &App) -> Column {
        self.columns.get(column).cloned().unwrap_or_default()
    }

    fn render_td(
        &mut self,
        row: usize,
        column: usize,
        _window: &mut Window,
        cx: &mut Context<TableState<Self>>,
    ) -> impl IntoElement {
        div()
            .w_full()
            .min_w_0()
            .truncate()
            .when(column == 0 || column == 5, |cell| {
                cell.text_color(cx.theme().muted_foreground)
            })
            .child(self.cell(row, column).to_owned())
    }

    fn cell_text(&self, row: usize, column: usize, _cx: &App) -> String {
        self.cell(row, column).to_owned()
    }
}

pub(crate) struct TableView {
    state: Entity<TableState<ResultsTable>>,
}

impl TableView {
    pub(crate) fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
        let state = cx.new(|cx| {
            let mut state = TableState::new(ResultsTable::sample(), window, cx)
                .row_selectable(true)
                .col_selectable(false);
            state.set_selected_row(2, cx);
            state
        });
        Self { state }
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

impl Render for TableView {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = cx.theme();
        let row_count = self.state.read(cx).delegate().rows.len();

        div()
            .flex_1()
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
                            .child(Self::toolbar_button(
                                "add-row",
                                "Row",
                                IconName::Plus,
                                "Add a row",
                            ))
                            .child(Self::toolbar_button(
                                "refresh-results",
                                "Refresh",
                                IconName::RefreshCw,
                                "Refresh results",
                            ))
                            .child(Self::toolbar_button(
                                "filter-results",
                                "Filter",
                                IconName::ListFilter,
                                "Filter results",
                            ))
                            .child(Self::toolbar_button(
                                "sort-results",
                                "Sort",
                                IconName::ArrowUpDown,
                                "Sort results",
                            )),
                    )
                    .child(
                        div()
                            .text_sm()
                            .text_color(theme.muted_foreground)
                            .child(format!("{row_count} rows  •  public.products")),
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sample_results_match_column_definitions() {
        let results = ResultsTable::sample();
        assert_eq!(results.rows.len(), 9);
        assert_eq!(results.columns.len(), 6);
        assert!(
            results
                .rows
                .iter()
                .all(|row| row.len() == results.columns.len())
        );
        assert_eq!(results.cell(2, 2), "USB-C Dock");
    }

    #[test]
    fn missing_cells_are_safe() {
        let results = ResultsTable::sample();
        assert_eq!(results.cell(usize::MAX, 0), "");
        assert_eq!(results.cell(0, usize::MAX), "");
    }
}
