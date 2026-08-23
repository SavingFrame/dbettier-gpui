use gpui::{Context, Div, Window, div, prelude::*, px};

use crate::theme::AppTheme;

pub(crate) struct TableView;

impl TableView {
    fn toolbar_button(label: &str, theme: &AppTheme) -> Div {
        div()
            .px_2()
            .py_1()
            .rounded_sm()
            .text_sm()
            .text_color(theme.text_muted)
            .child(label.to_owned())
    }

    fn table_cell(text: &str, width: f32, muted: bool, theme: &AppTheme) -> Div {
        div()
            .w(px(width))
            .flex_none()
            .px_3()
            .overflow_hidden()
            .truncate()
            .text_color(if muted { theme.text_muted } else { theme.text })
            .child(text.to_owned())
    }

    fn table_row(values: [&str; 6], selected: bool, theme: &AppTheme) -> Div {
        let row = div()
            .h(px(34.))
            .flex_none()
            .flex()
            .items_center()
            .border_b_1()
            .border_color(theme.border_subtle)
            .text_sm()
            .children([
                Self::table_cell(values[0], 56., true, theme),
                Self::table_cell(values[1], 110., false, theme),
                Self::table_cell(values[2], 260., false, theme),
                Self::table_cell(values[3], 170., false, theme),
                Self::table_cell(values[4], 120., false, theme),
                Self::table_cell(values[5], 150., true, theme),
            ]);

        if selected {
            row.bg(theme.selection)
        } else {
            row
        }
    }
}

impl Render for TableView {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = cx.global::<AppTheme>();
        let headers = [
            Self::table_cell("#", 56., true, theme),
            Self::table_cell("id", 110., false, theme),
            Self::table_cell("name", 260., false, theme),
            Self::table_cell("category", 170., false, theme),
            Self::table_cell("price", 120., false, theme),
            Self::table_cell("updated_at", 150., false, theme),
        ];
        let rows = [
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

        div()
            .flex_1()
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
                            .child(Self::toolbar_button("＋ Row", theme))
                            .child(Self::toolbar_button("↻ Refresh", theme))
                            .child(Self::toolbar_button("⌕ Filter", theme))
                            .child(Self::toolbar_button("⇅ Sort", theme)),
                    )
                    .child(
                        div()
                            .text_sm()
                            .text_color(theme.text_muted)
                            .child("9 rows  •  public.products"),
                    ),
            )
            .child(
                div()
                    .h(px(34.))
                    .flex_none()
                    .flex()
                    .items_center()
                    .bg(theme.panel_raised)
                    .border_b_1()
                    .border_color(theme.border)
                    .text_xs()
                    .font_weight(gpui::FontWeight::SEMIBOLD)
                    .children(headers),
            )
            .child(
                div().flex_1().overflow_hidden().children(
                    rows.into_iter()
                        .enumerate()
                        .map(|(index, row)| Self::table_row(row, index == 2, theme)),
                ),
            )
    }
}
