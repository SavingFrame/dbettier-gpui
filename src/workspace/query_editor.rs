use gpui::{Context, Window, div, prelude::*, px};

use crate::theme::AppTheme;

pub(crate) struct QueryEditor;

impl Render for QueryEditor {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = cx.global::<AppTheme>();
        let code_lines = [
            ("1", "SELECT id, name, category, price, updated_at"),
            ("2", "FROM public.products"),
            ("3", "WHERE category = 'Accessories'"),
            ("4", "ORDER BY updated_at DESC"),
            ("5", "LIMIT 100;"),
        ];

        div()
            .h(px(260.))
            .flex_none()
            .flex()
            .flex_col()
            .bg(theme.panel)
            .border_t_1()
            .border_color(theme.border)
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
                            .gap_3()
                            .child(div().text_color(theme.success).child("▶"))
                            .child(
                                div()
                                    .font_weight(gpui::FontWeight::SEMIBOLD)
                                    .child("query.sql"),
                            )
                            .child(
                                div()
                                    .text_sm()
                                    .text_color(theme.text_muted)
                                    .child("dbettier / public"),
                            ),
                    )
                    .child(
                        div()
                            .text_sm()
                            .text_color(theme.text_muted)
                            .child("Run  Ctrl+Enter"),
                    ),
            )
            .child(
                div()
                    .flex_1()
                    .p_3()
                    .font_family("monospace")
                    .text_sm()
                    .children(code_lines.into_iter().map(|(number, code)| {
                        div()
                            .flex()
                            .h_6()
                            .child(
                                div()
                                    .w(px(36.))
                                    .flex_none()
                                    .text_color(theme.line_number)
                                    .child(number),
                            )
                            .child(div().text_color(theme.editor_text).child(code))
                    })),
            )
    }
}
