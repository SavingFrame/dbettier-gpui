use gpui::{Context, Div, Entity, Subscription, Window, div, prelude::*, px};

use crate::{database::profile_store::DatabaseProfileStore, theme::AppTheme};

pub(crate) struct DatabaseTree {
    profile_store: Entity<DatabaseProfileStore>,
    _profile_store_subscription: Subscription,
}

impl DatabaseTree {
    pub(crate) fn new(cx: &mut Context<Self>) -> Self {
        let profile_store = DatabaseProfileStore::global(cx);
        let profile_store_subscription = cx.observe(&profile_store, |_, _, cx| cx.notify());

        Self {
            profile_store,
            _profile_store_subscription: profile_store_subscription,
        }
    }

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
        let profile_store = self.profile_store.read(cx);
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
            .child(
                div()
                    .flex_1()
                    .overflow_hidden()
                    .p_2()
                    .when(profile_store.profiles().is_empty(), |tree| {
                        tree.child(
                            div()
                                .p_2()
                                .text_sm()
                                .text_color(theme.text_muted)
                                .child("No database profiles. Use + to add one."),
                        )
                    })
                    .children(profile_store.profiles().iter().map(|profile| {
                        Self::render_row(&format!("▸  {}", profile.name), 0., false, theme)
                    })),
            )
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
