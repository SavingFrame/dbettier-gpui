use gpui::{Context, Window, div, prelude::*, px};

use gpui_kit::{
    assets::IconName,
    component::{
        ActiveTheme as _, Sizable as _,
        button::{Button, ButtonVariants as _},
    },
};

pub(crate) struct QueryEditor;

impl Render for QueryEditor {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = cx.theme();
        let code_lines = [
            ("1", "SELECT id, name, category, price, updated_at"),
            ("2", "FROM public.products"),
            ("3", "WHERE category = 'Accessories'"),
            ("4", "ORDER BY updated_at DESC"),
            ("5", "LIMIT 100;"),
        ];

        div()
            .size_full()
            .flex()
            .flex_col()
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
                            .gap_3()
                            .child(
                                div()
                                    .font_weight(gpui::FontWeight::SEMIBOLD)
                                    .child("query.sql"),
                            )
                            .child(
                                div()
                                    .text_sm()
                                    .text_color(theme.muted_foreground)
                                    .child("dbettier / public"),
                            ),
                    )
                    .child(
                        Button::new("run-query")
                            .ghost()
                            .small()
                            .icon(IconName::Play)
                            .label("Run")
                            .tooltip("Run query (execution not implemented yet)"),
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
                                    .text_color(theme.muted_foreground)
                                    .child(number),
                            )
                            .child(div().text_color(theme.foreground).child(code))
                    })),
            )
    }
}
