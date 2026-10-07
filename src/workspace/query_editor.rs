use gpui::{App, Context, FocusHandle, Focusable, Window, div, prelude::*, px};

use gpui_kit::{
    assets::IconName,
    component::{
        ActiveTheme as _, Sizable as _,
        button::{Button, ButtonVariants as _},
        dock::{BasePanel, Panel, PanelEvent},
    },
};

pub(crate) struct QueryEditor {
    focus_handle: FocusHandle,
}

impl QueryEditor {
    pub(crate) fn new(cx: &mut Context<Self>) -> Self {
        Self {
            focus_handle: cx.focus_handle(),
        }
    }
}

impl Focusable for QueryEditor {
    fn focus_handle(&self, _: &App) -> FocusHandle {
        self.focus_handle.clone()
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
            .track_focus(&self.focus_handle)
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
