use gpui::{Context, Div, Window, div, prelude::*, px};

use crate::theme::AppTheme;

pub(crate) struct DatabaseTree;

impl DatabaseTree {
    fn render_row(label: &str, depth: f32, selected: bool, theme: &AppTheme) -> Div {
        let row = div()
            .flex()
            .items_center()
            .h_7()
            .pl(px(12. + depth * 16.))
            .pr_2()
            .rounded_sm()
            .text_sm()
            .text_color(if selected {
                theme.text
            } else {
                theme.text_subtle
            })
            .child(label.to_owned());

        if selected {
            row.bg(theme.selection)
        } else {
            row
        }
    }
}

impl Render for DatabaseTree {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = cx.global::<AppTheme>();

        div()
            .w(px(280.))
            .flex_none()
            .flex()
            .flex_col()
            .bg(theme.panel)
            .border_r_1()
            .border_color(theme.border)
            .child(
                div()
                    .h(px(44.))
                    .flex_none()
                    .flex()
                    .items_center()
                    .justify_between()
                    .px_3()
                    .border_b_1()
                    .border_color(theme.border)
                    .child(
                        div()
                            .font_weight(gpui::FontWeight::SEMIBOLD)
                            .child("Database"),
                    )
                    .child(div().text_color(theme.text_muted).child("+   ↻   ⋯")),
            )
            .child(div().flex_1().overflow_hidden().p_2().children([
                Self::render_row("▾  local development", 0., false, theme),
                Self::render_row("▾  dbettier", 1., false, theme),
                Self::render_row("▾  Schemas", 2., false, theme),
                Self::render_row("▾  public", 3., false, theme),
                Self::render_row("▾  Tables  8", 4., false, theme),
                Self::render_row("▦  customers", 5., false, theme),
                Self::render_row("▦  invoices", 5., false, theme),
                Self::render_row("▦  invoice_items", 5., false, theme),
                Self::render_row("▦  products", 5., true, theme),
                Self::render_row("▦  suppliers", 5., false, theme),
                Self::render_row("▦  users", 5., false, theme),
                Self::render_row("▸  Views  3", 4., false, theme),
                Self::render_row("▸  Functions  12", 4., false, theme),
                Self::render_row("▸  Sequences  5", 4., false, theme),
                Self::render_row("▸  Extensions", 3., false, theme),
            ]))
            .child(
                div()
                    .h(px(38.))
                    .flex_none()
                    .flex()
                    .items_center()
                    .gap_2()
                    .px_3()
                    .border_t_1()
                    .border_color(theme.border)
                    .text_sm()
                    .text_color(theme.text_muted)
                    .child("●")
                    .child("PostgreSQL 16"),
            )
    }
}
