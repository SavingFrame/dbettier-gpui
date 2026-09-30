use gpui::{Context, Window, div, prelude::*, px};

use gpui_kit::component::ActiveTheme as _;

pub(crate) struct StatusBar;

impl Render for StatusBar {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = cx.theme();

        div()
            .h(px(28.))
            .flex_none()
            .flex()
            .items_center()
            .justify_between()
            .px_3()
            .bg(theme.muted)
            .border_t_1()
            .border_color(theme.border)
            .text_xs()
            .text_color(theme.muted_foreground)
            .child("● Connected  localhost:5432 / dbettier")
            .child("PostgreSQL  •  UTF-8  •  LF  •  9 rows")
    }
}
