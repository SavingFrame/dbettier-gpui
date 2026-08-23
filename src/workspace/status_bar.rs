use gpui::{Context, Window, div, prelude::*, px};

use crate::theme::AppTheme;

pub(crate) struct StatusBar;

impl Render for StatusBar {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = cx.global::<AppTheme>();

        div()
            .h(px(28.))
            .flex_none()
            .flex()
            .items_center()
            .justify_between()
            .px_3()
            .bg(theme.status_bar)
            .border_t_1()
            .border_color(theme.border)
            .text_xs()
            .text_color(theme.text_muted)
            .child("● Connected  localhost:5432 / dbettier")
            .child("PostgreSQL  •  UTF-8  •  LF  •  9 rows")
    }
}
