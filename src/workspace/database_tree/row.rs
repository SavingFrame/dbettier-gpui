use gpui::{App, MouseButton, SharedString, WeakEntity, div, prelude::*, px, svg};
use gpui_kit::component::{
    ActiveTheme as _,
    list::ListItem,
    menu::{ContextMenuExt as _, PopupMenuItem},
    tree::TreeEntry,
};

use crate::workspace::{Target, database_tree::SqlTarget};

use super::{
    DatabaseTree,
    model::{RowAction, RowInfo},
};

#[derive(Debug, PartialEq, Eq)]
pub(super) enum RowInteraction {
    Select,
    Toggle,
    OpenTable,
}

impl RowAction {
    pub(super) fn interaction(&self, arrow: bool, click_count: usize) -> RowInteraction {
        if arrow {
            if click_count == 1 {
                RowInteraction::Toggle
            } else {
                RowInteraction::Select
            }
        } else if click_count == 2 {
            if matches!(self, Self::Table(_)) {
                RowInteraction::OpenTable
            } else {
                RowInteraction::Toggle
            }
        } else {
            RowInteraction::Select
        }
    }
}

/// Renders a ListItem because the library tree requires that concrete type.
pub(super) struct DatabaseTreeRow<'a> {
    id: SharedString,
    label: SharedString,
    depth: usize,
    folder: bool,
    expanded: bool,
    selected: bool,
    info: Option<&'a RowInfo>,
    view: WeakEntity<DatabaseTree>,
}

impl<'a> DatabaseTreeRow<'a> {
    pub(super) fn new(
        entry: &TreeEntry,
        selected: bool,
        info: Option<&'a RowInfo>,
        view: WeakEntity<DatabaseTree>,
    ) -> Self {
        Self {
            id: entry.item().id.clone(),
            label: entry.item().label.clone(),
            depth: entry.depth(),
            folder: entry.is_folder(),
            expanded: entry.is_expanded(),
            selected,
            info,
            view,
        }
    }

    pub(super) fn render(self, cx: &mut App) -> ListItem {
        let theme = cx.theme();
        let row = self.info.as_ref();
        let icon = row.and_then(|row| row.icon);
        let status = row
            .and_then(|row| row.status)
            .map(|status| status.color(theme));
        let color = if row.is_some_and(|row| row.error) {
            theme.danger
        } else {
            theme.sidebar_foreground
        };
        let id = self.id.clone();
        let select_id = id.clone();
        let select_view = self.view.clone();
        let activate_view = self.view.clone();
        let arrow_id = id.clone();
        let arrow_select_id = id.clone();
        let arrow_view = self.view.clone();
        let arrow_select_view = self.view.clone();
        let context_id = id.clone();
        let context_view = self.view.clone();

        let sql_target = row.and_then(|row| match &row.action {
            Some(RowAction::Schema { profile_id, name }) => Some(SqlTarget {
                profile_id: profile_id.clone(),
                schema_name: Some(name.clone()),
            }),
            _ => None,
        });
        ListItem::new(id.clone())
            .on_mouse_down(MouseButton::Left, move |_, window, cx| {
                // The library tree toggles on mouse-down unless we intercept it.
                cx.stop_propagation();
                if let Some(view) = select_view.upgrade() {
                    view.update(cx, |view, cx| view.select_row(&select_id, window, cx));
                }
            })
            .on_click(move |event, _, cx| {
                cx.stop_propagation();
                if let Some(view) = activate_view.upgrade() {
                    view.update(cx, |view, cx| {
                        view.activate_row(&id, false, event.click_count(), cx)
                    });
                }
            })
            .selected(self.selected)
            .h_7()
            .pl(px(12. + self.depth as f32 * 16.))
            .text_sm()
            .text_color(color)
            .child(
                div()
                    .id(self.id.clone())
                    .flex()
                    .items_center()
                    .size_full()
                    .min_w_0()
                    .child(
                        div()
                            .id("disclosure")
                            .w_4()
                            .flex_none()
                            .when(self.folder, |arrow| {
                                arrow
                                    .cursor_pointer()
                                    .on_mouse_down(MouseButton::Left, move |_, window, cx| {
                                        cx.stop_propagation();
                                        if let Some(view) = arrow_select_view.upgrade() {
                                            view.update(cx, |view, cx| {
                                                view.select_row(&arrow_select_id, window, cx)
                                            });
                                        }
                                    })
                                    .on_click(move |event, _, cx| {
                                        cx.stop_propagation();
                                        if let Some(view) = arrow_view.upgrade() {
                                            view.update(cx, |view, cx| {
                                                view.activate_row(
                                                    &arrow_id,
                                                    true,
                                                    event.click_count(),
                                                    cx,
                                                )
                                            });
                                        }
                                    })
                            })
                            .text_color(theme.muted_foreground)
                            .child(if self.folder {
                                if self.expanded { "▾" } else { "▸" }
                            } else {
                                ""
                            }),
                    )
                    .when_some(icon, |row, icon| {
                        row.child(
                            svg()
                                .path(icon)
                                .size_4()
                                .flex_none()
                                .mr_2()
                                .text_color(theme.muted_foreground),
                        )
                    })
                    .child(
                        div()
                            .min_w_0()
                            .flex_1()
                            .overflow_hidden()
                            .whitespace_nowrap()
                            .text_ellipsis()
                            .child(self.label.clone()),
                    )
                    .when_some(status, |row, status| {
                        row.child(
                            div()
                                .flex_none()
                                .w(px(7.))
                                .h(px(7.))
                                .rounded_full()
                                .bg(status),
                        )
                    })
                    .context_menu(move |menu, window, cx| {
                        let Some(target) = sql_target.clone() else {
                            return menu;
                        };
                        if let Some(view) = context_view.upgrade() {
                            view.update(cx, |view, cx| {
                                view.select_row(&context_id, window, cx);
                            });
                        }
                        let console_view = context_view.clone();
                        menu.item(
                            PopupMenuItem::new("Open console").on_click(move |_, _, cx| {
                                if let Some(view) = console_view.upgrade() {
                                    view.update(cx, |view, cx| {
                                        view.on_open_table(Target::Sql(target.clone()), cx);
                                    });
                                }
                            }),
                        )
                    }),
            )
    }
}

#[cfg(test)]
mod tests {
    use super::super::model::TableTarget;
    use super::*;

    #[test]
    fn row_clicks_select_and_double_clicks_activate() {
        let table = RowAction::Table(TableTarget {
            profile_id: "profile".to_owned(),
            schema_name: "public".to_owned(),
            table_name: "products".to_owned(),
        });
        for action in [&table, &RowAction::Section] {
            assert_eq!(action.interaction(false, 1), RowInteraction::Select);
            assert_eq!(action.interaction(false, 3), RowInteraction::Select);
            assert_eq!(action.interaction(true, 1), RowInteraction::Toggle);
            assert_eq!(action.interaction(true, 2), RowInteraction::Select);
        }
        assert_eq!(table.interaction(false, 2), RowInteraction::OpenTable);
        assert_eq!(
            RowAction::Section.interaction(false, 2),
            RowInteraction::Toggle
        );
        let schema = RowAction::Schema {
            profile_id: "profile".to_owned(),
            name: "public".to_owned(),
        };
        assert_eq!(schema.interaction(false, 2), RowInteraction::Toggle);
    }
}
