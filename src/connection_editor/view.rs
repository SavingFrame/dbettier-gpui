use gpui::{Context, Entity, SharedString, Window, div, prelude::*, px};
use gpui_kit::component::{
    ActiveTheme as _, Disableable as _, Selectable as _,
    button::{Button, ButtonVariants as _},
    input::{Input, InputState},
    select::Select,
};

use super::ConnectionEditor;
use super::model::ConnectionInputMode;

impl ConnectionEditor {
    fn render_footer(&self, cx: &Context<Self>) -> gpui::AnyElement {
        // let busy = self.status.busy();
        let busy = false; // TODO:
        div()
            .flex()
            .items_center()
            .justify_between()
            .gap_2()
            .px_6()
            .py_4()
            .flex_none()
            .border_t_1()
            .border_color(cx.theme().border)
            .child(
                Button::new("test-connection")
                    .on_click(cx.listener(|view, _, _, cx| {
                        view.draft(cx);
                    }))
                    // .label(if matches!(self.status, ConnectionEditorStatus::Testing) {
                    //     "Testing..."
                    // } else {
                    //     "Test Connection"
                    // })
                    // .loading(matches!(self.status, ConnectionEditorStatus::Testing))
                    .disabled(busy)
                    .label("Test connection"), // .on_click(cx.listener(|editor, _, _, cx| editor.request_test(cx))),
            )
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap_2()
                    .child(Button::new("cancel-connection").ghost().label("Cancel"))
                    .child(
                        Button::new("save-connection")
                            .primary()
                            .label(if self.profile_id.is_some() {
                                "Save Changes"
                            } else {
                                "Add Connection"
                            })
                            // .loading(matches!(self.status, ConnectionEditorStatus::Saving))
                            .disabled(busy),
                    ),
            )
            .into_any_element()
    }

    fn field(
        &self,
        label: &'static str,
        state: &Entity<InputState>,
        secret: bool,
    ) -> impl IntoElement {
        let busy = false; // TODO:
        div()
            .flex()
            .flex_col()
            .flex_1()
            .min_w_0()
            .gap_1()
            .child(div().text_sm().child(label))
            .child(
                Input::new(state)
                    .aria_label(label)
                    .disabled(busy)
                    .when(secret, |input| input.mask_toggle()),
            )
    }
}

impl ConnectionEditor {
    fn render_form(&self, cx: &Context<Self>) -> gpui::AnyElement {
        let theme = cx.theme();
        let busy = false; // TODO:
        let message = "Test the connection before saving (optional).".to_string();
        let color = theme.muted_foreground;
        // let (message, color) = match &self.status {
        //     ConnectionEditorStatus::Idle => (
        //         "Test the connection before saving (optional).",
        //         theme.muted_foreground,
        //     ),
        //     ConnectionEditorStatus::Testing => ("Testing connection...", theme.muted_foreground),
        //     ConnectionEditorStatus::Saving => ("Saving connection...", theme.muted_foreground),
        //     ConnectionEditorStatus::Success(message) => (message.as_str(), theme.success),
        //     ConnectionEditorStatus::Error(message) => (message.as_str(), theme.danger),
        //     ConnectionEditorStatus::Notice(message) => (message.as_str(), theme.muted_foreground),
        // };
        div()
            .flex()
            .flex_col()
            .gap_4()
            .pb_4()
            .child(
                div()
                    .flex()
                    .items_center()
                    .justify_between()
                    .gap_2()
                    .child(
                        div()
                            .text_sm()
                            .text_color(theme.muted_foreground)
                            .child("PostgreSQL"),
                    )
                    .child(
                        div()
                            .text_xs()
                            .text_color(theme.muted_foreground)
                            .child("Connection profile"),
                    ),
            )
            .child(self.field("Connection name", &self.name, false))
            .child(
                div()
                    .flex()
                    .flex_col()
                    .gap_2()
                    .child(
                        div()
                            .flex()
                            .gap_1()
                            .p_1()
                            .rounded_md()
                            .bg(theme.muted)
                            .children(
                                [
                                    (ConnectionInputMode::Fields, "Connection fields"),
                                    (ConnectionInputMode::Uri, "Connection URI"),
                                ]
                                .into_iter()
                                .enumerate()
                                .map(|(index, (mode, label))| {
                                    Button::new(("connection-mode", index))
                                        .ghost()
                                        .flex_1()
                                        .label(label)
                                        .selected(self.mode == mode)
                                        .disabled(busy)
                                        .on_click(cx.listener(move |editor, _, window, cx| {
                                            editor.mode = mode;
                                            let state = match mode {
                                                ConnectionInputMode::Fields => &editor.host,
                                                ConnectionInputMode::Uri => &editor.uri,
                                            };
                                            state.update(cx, |state, cx| state.focus(window, cx));
                                            cx.notify();
                                        }))
                                }),
                            ),
                    )
                    .child(
                        div().text_xs().text_color(theme.muted_foreground).child(
                            "Each mode keeps its own values. Only the selected mode is used.",
                        ),
                    ),
            )
            .when(self.mode == ConnectionInputMode::Fields, |form| {
                let ssl_mode = self
                    .ssl_mode
                    .read(cx)
                    .selected_value()
                    .copied()
                    .unwrap_or("prefer");
                form.child(
                    div()
                        .flex()
                        .gap_3()
                        .child(self.field("Host", &self.host, false))
                        .child(
                            div()
                                .w(px(110.))
                                .child(self.field("Port", &self.port, false)),
                        ),
                )
                .child(self.field("Database", &self.database, false))
                .child(
                    div()
                        .flex()
                        .gap_3()
                        .child(self.field("Username", &self.username, false))
                        .child(self.field("Password", &self.password, true)),
                )
                .child(
                    div()
                        .flex()
                        .flex_col()
                        .gap_1()
                        .child(div().text_sm().child("SSL mode"))
                        .child(
                            Select::new(&self.ssl_mode)
                                .accessibility_label("SSL mode")
                                .disabled(busy),
                        )
                        .child(
                            div()
                                .text_xs()
                                .text_color(theme.muted_foreground)
                                .child(ssl_description(ssl_mode)),
                        ),
                )
            })
            .when(self.mode == ConnectionInputMode::Uri, |form| {
                form.child(self.field("Connection URI", &self.uri, true))
                    .child(
                        div()
                            .flex()
                            .flex_col()
                            .gap_1()
                            .text_xs()
                            .text_color(theme.muted_foreground)
                            .child("Include SSL settings in the URI, for example ?sslmode=require.")
                            .child("The URI is masked because it may contain a password."),
                    )
            })
            .child(
                div()
                    .rounded_md()
                    .bg(theme.muted)
                    .p_3()
                    .text_sm()
                    .text_color(color)
                    .child(SharedString::from(message.to_owned())),
            )
            .into_any_element()
    }
}

impl Render for ConnectionEditor {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = cx.theme();
        div()
            .key_context("ConnectionEditor")
            .track_focus(&self.focus_handle)
            .tab_group()
            .size_full()
            .flex()
            .flex_col()
            .overflow_hidden()
            .bg(theme.background)
            .text_color(theme.foreground)
            .child(
                div()
                    .flex_none()
                    .px_6()
                    .py_5()
                    .border_b_1()
                    .border_color(theme.border)
                    .child(
                        div()
                            .text_lg()
                            .font_weight(gpui::FontWeight::SEMIBOLD)
                            .child(if self.profile_id.is_some() {
                                "Edit database connection"
                            } else {
                                "Add database connection"
                            }),
                    )
                    .child(
                        div()
                            .mt_1()
                            .text_sm()
                            .text_color(theme.muted_foreground)
                            .child("Configure a PostgreSQL connection."),
                    ),
            )
            .child(
                div()
                    .id("connection-form")
                    .flex_1()
                    .min_h_0()
                    .overflow_y_scroll()
                    .p_6()
                    .child(self.render_form(cx)),
            )
            .child(self.render_footer(cx))
    }
}

fn ssl_description(mode: &str) -> &'static str {
    match mode {
        "disable" => "Do not use SSL. Traffic is not encrypted.",
        "allow" => "Try without SSL first, then use SSL if required.",
        "require" => "Require an encrypted connection.",
        "verify-ca" => "Require SSL and verify the server certificate authority.",
        "verify-full" => "Require SSL and verify the certificate authority and hostname.",
        _ => "Prefer SSL, but allow an unencrypted connection if unavailable.",
    }
}
