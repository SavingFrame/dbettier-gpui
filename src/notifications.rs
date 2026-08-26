use std::time::Duration;

use gpui::{Context, Window, div, prelude::*, px};
use uuid::Uuid;

use crate::theme::AppTheme;

const NOTIFICATION_DURATION: Duration = Duration::from_secs(10);
const MAX_VISIBLE_NOTIFICATIONS: usize = 3;

#[derive(Clone)]
struct Notification {
    key: String,
    instance_id: Uuid,
    title: String,
    message: String,
}

pub(crate) struct NotificationCenter {
    notifications: Vec<Notification>,
}

impl NotificationCenter {
    pub(crate) fn new() -> Self {
        Self {
            notifications: Vec::new(),
        }
    }

    pub(crate) fn show_error(
        &mut self,
        key: impl Into<String>,
        title: impl Into<String>,
        message: impl Into<String>,
        cx: &mut Context<Self>,
    ) {
        let key = key.into();
        let instance_id = Uuid::new_v4();

        self.notifications
            .retain(|notification| notification.key != key);
        self.notifications.push(Notification {
            key,
            instance_id,
            title: title.into(),
            message: message.into(),
        });

        if self.notifications.len() > MAX_VISIBLE_NOTIFICATIONS {
            self.notifications.remove(0);
        }

        cx.notify();

        cx.spawn(async move |notification_center, cx| {
            cx.background_executor().timer(NOTIFICATION_DURATION).await;

            if let Err(error) = notification_center.update(cx, |notification_center, cx| {
                notification_center.dismiss(instance_id, cx);
            }) {
                eprintln!("failed to dismiss notification: {error}");
            }
        })
        .detach();
    }

    fn dismiss(&mut self, instance_id: Uuid, cx: &mut Context<Self>) {
        let previous_len = self.notifications.len();
        self.notifications
            .retain(|notification| notification.instance_id != instance_id);

        if self.notifications.len() != previous_len {
            cx.notify();
        }
    }
}

impl Render for NotificationCenter {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = cx.global::<AppTheme>();

        div()
            .absolute()
            .top_4()
            .right_4()
            .w(px(420.))
            .flex()
            .flex_col()
            .gap_2()
            .children(self.notifications.clone().into_iter().map(|notification| {
                let instance_id = notification.instance_id;

                div()
                    .id(format!("notification-{instance_id}"))
                    .flex()
                    .items_start()
                    .gap_3()
                    .p_3()
                    .rounded_md()
                    .border_1()
                    .border_color(theme.error)
                    .bg(theme.panel_raised)
                    .shadow_lg()
                    .child(
                        div()
                            .mt_1()
                            .size(px(8.))
                            .flex_none()
                            .rounded_full()
                            .bg(theme.error),
                    )
                    .child(
                        div()
                            .min_w_0()
                            .flex_1()
                            .flex()
                            .flex_col()
                            .gap_1()
                            .child(
                                div()
                                    .text_sm()
                                    .font_weight(gpui::FontWeight::SEMIBOLD)
                                    .text_color(theme.text)
                                    .child(notification.title),
                            )
                            .child(
                                div()
                                    .text_sm()
                                    .whitespace_normal()
                                    .text_color(theme.text_subtle)
                                    .child(notification.message),
                            ),
                    )
                    .child(
                        div()
                            .id(format!("dismiss-notification-{instance_id}"))
                            .flex_none()
                            .px_1()
                            .rounded_sm()
                            .cursor_pointer()
                            .text_color(theme.text_muted)
                            .child("×")
                            .on_click(cx.listener(move |notification_center, _, _, cx| {
                                notification_center.dismiss(instance_id, cx);
                                cx.stop_propagation();
                            })),
                    )
            }))
    }
}
