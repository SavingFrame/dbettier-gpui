mod labels;

use std::{
    collections::{HashMap, HashSet},
    rc::Rc,
};

use gpui::{
    Context, Entity, Hsla, MouseButton, SharedString, Subscription, Window, div, prelude::*, px,
    svg,
};
use gpui_kit::assets::IconName;
use gpui_kit::component::{
    ActiveTheme as _, Sizable as _, Theme,
    button::{Button, ButtonVariants as _},
    list::ListItem,
    tree::{TreeEvent, TreeItem, TreeState, tree},
};

use crate::database::{
    DatabaseConnectionProfile, DatabaseSchema, DatabaseTable, LoadState,
    profile_store::DatabaseProfileStore, session::DatabaseSessionState,
    session_store::DatabaseSessionStore,
};

#[derive(Clone, Copy)]
enum TableSection {
    Columns,
    Constraints,
    Indexes,
}

impl TableSection {
    fn name(self) -> &'static str {
        match self {
            Self::Columns => "columns",
            Self::Constraints => "constraints",
            Self::Indexes => "indexes",
        }
    }

    fn title(self) -> &'static str {
        match self {
            Self::Columns => "Columns",
            Self::Constraints => "Constraints",
            Self::Indexes => "Indexes",
        }
    }

    fn icon(self) -> &'static str {
        match self {
            Self::Columns => "icons/column.svg",
            Self::Constraints => "icons/key.svg",
            Self::Indexes => "icons/index.svg",
        }
    }
}

#[derive(Clone)]
enum RowAction {
    Profile(DatabaseConnectionProfile),
    Schema { profile_id: String, name: String },
    Table(TableTarget),
    Section,
}

#[derive(Debug, PartialEq, Eq)]
enum RowInteraction {
    Select,
    Toggle,
    OpenTable,
}

impl RowAction {
    fn interaction(&self, arrow: bool, click_count: usize) -> RowInteraction {
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

#[derive(Clone)]
struct TableTarget {
    profile_id: String,
    schema_name: String,
    table_name: String,
}

#[derive(Clone, Copy)]
enum ConnectionStatus {
    Disconnected,
    Connecting,
    Failed,
    Connected,
}

impl ConnectionStatus {
    fn color(self, theme: &Theme) -> Hsla {
        match self {
            Self::Disconnected => theme.muted_foreground,
            Self::Connecting => theme.warning,
            Self::Failed => theme.danger,
            Self::Connected => theme.success,
        }
    }
}

struct RowInfo {
    action: Option<RowAction>,
    icon: Option<&'static str>,
    status: Option<ConnectionStatus>,
    error: bool,
}

impl RowInfo {
    fn folder(
        action: RowAction,
        icon: Option<&'static str>,
        status: Option<ConnectionStatus>,
    ) -> Self {
        Self {
            action: Some(action),
            icon,
            status,
            error: false,
        }
    }

    fn leaf(icon: Option<&'static str>, error: bool) -> Self {
        Self {
            action: None,
            icon,
            status: None,
            error,
        }
    }
}

pub(crate) struct DatabaseTree {
    profile_store: Entity<DatabaseProfileStore>,
    session_store: Entity<DatabaseSessionStore>,
    tree_state: Entity<TreeState>,
    rows: Rc<HashMap<SharedString, RowInfo>>,
    expanded_items: HashSet<String>,
    rebuild_scheduled: bool,
    _profile_store_subscription: Subscription,
    _session_store_subscription: Subscription,
    _tree_subscription: Subscription,
}

impl DatabaseTree {
    pub(crate) fn new(cx: &mut Context<Self>) -> Self {
        let profile_store = DatabaseProfileStore::global(cx);
        let session_store = DatabaseSessionStore::global(cx);
        let tree_state = cx.new(|cx| TreeState::new(cx));
        let profile_store_subscription = cx.observe(&profile_store, |tree, _, cx| {
            tree.schedule_rebuild(cx);
        });
        let session_store_subscription = cx.observe(&session_store, |tree, _, cx| {
            tree.schedule_rebuild(cx);
        });
        let tree_subscription = cx.subscribe(&tree_state, |tree, _, event: &TreeEvent, cx| {
            tree.on_tree_event(event, cx);
        });

        let mut tree = Self {
            profile_store,
            session_store,
            tree_state,
            rows: Rc::new(HashMap::new()),
            expanded_items: HashSet::new(),
            rebuild_scheduled: false,
            _profile_store_subscription: profile_store_subscription,
            _session_store_subscription: session_store_subscription,
            _tree_subscription: tree_subscription,
        };
        tree.rebuild(cx);
        tree
    }

    fn schedule_rebuild(&mut self, cx: &mut Context<Self>) {
        if self.rebuild_scheduled {
            return;
        }
        self.rebuild_scheduled = true;
        let entity = cx.entity();
        cx.defer(move |cx| {
            entity.update(cx, |tree, cx| {
                tree.rebuild_scheduled = false;
                tree.rebuild(cx);
            });
        });
    }

    fn on_tree_event(&mut self, event: &TreeEvent, cx: &mut Context<Self>) {
        let (id, expanded) = match event {
            TreeEvent::Expanded(id) => (id, true),
            TreeEvent::Collapsed(id) => (id, false),
        };
        let Some(action) = self.rows.get(id).and_then(|row| row.action.clone()) else {
            return;
        };

        if expanded {
            self.expanded_items.insert(id.to_string());
        } else {
            self.expanded_items.remove(id.as_ref());
            return;
        }

        match action {
            RowAction::Profile(profile) => {
                self.session_store
                    .update(cx, |store, cx| store.connect(profile, cx));
            }
            RowAction::Schema { profile_id, name } => {
                let session = self.session_store.read(cx).session(&profile_id).cloned();
                if let Some(session) = session {
                    session.update(cx, |session, cx| session.load_tables(&name, cx));
                }
            }
            RowAction::Table(target) => {
                let key =
                    Self::table_key(&target.profile_id, &target.schema_name, &target.table_name);
                for section in [
                    TableSection::Columns,
                    TableSection::Constraints,
                    TableSection::Indexes,
                ] {
                    self.expanded_items
                        .insert(format!("section:{}", Self::section_key(&key, section)));
                }
                self.schedule_rebuild(cx);
            }
            RowAction::Section => {}
        }
    }

    fn select_row(&mut self, id: &SharedString, window: &mut Window, cx: &mut Context<Self>) {
        self.tree_state.update(cx, |state, cx| {
            state.set_selected_index(state.index_of(id), cx);
            state.focus(window, cx);
        });
    }

    fn activate_row(
        &mut self,
        id: &SharedString,
        arrow: bool,
        click_count: usize,
        cx: &mut Context<Self>,
    ) {
        let Some(action) = self.rows.get(id).and_then(|row| row.action.clone()) else {
            return;
        };
        match action.interaction(arrow, click_count) {
            RowInteraction::Select => return,
            RowInteraction::OpenTable => {
                if let RowAction::Table(target) = action {
                    self.on_open_table(&target);
                }
                return;
            }
            RowInteraction::Toggle => {}
        }
        let event = if self.expanded_items.contains(id.as_ref()) {
            TreeEvent::Collapsed(id.clone())
        } else {
            TreeEvent::Expanded(id.clone())
        };
        self.on_tree_event(&event, cx);
        self.rebuild(cx);
    }

    fn on_open_table(&mut self, target: &TableTarget) {
        eprintln!(
            "Open table: profile={}, schema={}, table={}",
            target.profile_id, target.schema_name, target.table_name,
        );
    }

    fn schema_key(profile_id: &str, schema_name: &str) -> String {
        format!("{profile_id}:{}:{schema_name}", schema_name.len())
    }

    fn table_key(profile_id: &str, schema_name: &str, table_name: &str) -> String {
        format!(
            "{}:{}:{table_name}",
            Self::schema_key(profile_id, schema_name),
            table_name.len()
        )
    }

    fn section_key(table_key: &str, section: TableSection) -> String {
        format!("{table_key}:{}", section.name())
    }

    fn add_row(
        rows: &mut HashMap<SharedString, RowInfo>,
        id: String,
        label: impl Into<SharedString>,
        info: RowInfo,
        children: Vec<TreeItem>,
        expanded: bool,
    ) -> TreeItem {
        let id: SharedString = id.into();
        rows.insert(id.clone(), info);
        TreeItem::new(id, label)
            .children(children)
            .expanded(expanded)
    }

    fn message(
        id: &str,
        label: &str,
        rows: &mut HashMap<SharedString, RowInfo>,
        error: bool,
    ) -> TreeItem {
        let id: SharedString = format!("{id}:message").into();
        rows.insert(id.clone(), RowInfo::leaf(None, error));
        TreeItem::new(id, label).disabled(true)
    }

    fn rebuild(&mut self, cx: &mut Context<Self>) {
        let profiles = self.profile_store.read(cx).profiles().to_vec();
        let mut rows = HashMap::new();
        let items: Vec<TreeItem> = profiles
            .iter()
            .map(|profile| self.profile_item(profile, &mut rows, cx))
            .collect();
        self.rows = Rc::new(rows);
        self.tree_state.update(cx, |state, cx| {
            let selected = state.selected_item().map(|item| item.id.clone());
            state.set_items(items, cx);
            if let Some(id) = selected {
                state.set_selected_index(state.index_of(&id), cx);
            }
        });
        cx.notify();
    }

    fn profile_item(
        &self,
        profile: &DatabaseConnectionProfile,
        rows: &mut HashMap<SharedString, RowInfo>,
        cx: &mut Context<Self>,
    ) -> TreeItem {
        let id = format!("profile:{}", profile.uuid);
        let session = self.session_store.read(cx).session(&profile.uuid).cloned();
        let (status, children) = match session.as_ref().map(|session| session.read(cx).state()) {
            None | Some(DatabaseSessionState::Disconnected) => (
                ConnectionStatus::Disconnected,
                vec![Self::message(&id, "Starting connection...", rows, false)],
            ),
            Some(DatabaseSessionState::Connecting) => (
                ConnectionStatus::Connecting,
                vec![Self::message(&id, "Connecting...", rows, false)],
            ),
            Some(DatabaseSessionState::Failed(error)) => (
                ConnectionStatus::Failed,
                vec![Self::message(&id, error, rows, true)],
            ),
            Some(DatabaseSessionState::Connected { schemas, .. }) => {
                let children = match schemas {
                    LoadState::NotLoaded | LoadState::Loading => {
                        vec![Self::message(&id, "Loading schemas...", rows, false)]
                    }
                    LoadState::Failed(error) => vec![Self::message(&id, error, rows, true)],
                    LoadState::Loaded(schemas) if schemas.is_empty() => {
                        vec![Self::message(&id, "No schemas", rows, false)]
                    }
                    LoadState::Loaded(schemas) => schemas
                        .iter()
                        .map(|schema| self.schema_item(&profile.uuid, schema, rows))
                        .collect(),
                };
                (ConnectionStatus::Connected, children)
            }
        };
        Self::add_row(
            rows,
            id,
            profile.name.clone(),
            RowInfo::folder(
                RowAction::Profile(profile.clone()),
                Some("icons/database.svg"),
                Some(status),
            ),
            children,
            self.expanded_items
                .contains(&format!("profile:{}", profile.uuid)),
        )
    }

    fn schema_item(
        &self,
        profile_id: &str,
        schema: &DatabaseSchema,
        rows: &mut HashMap<SharedString, RowInfo>,
    ) -> TreeItem {
        let key = Self::schema_key(profile_id, &schema.name);
        let id = format!("schema:{key}");
        let children = match &schema.tables {
            LoadState::NotLoaded | LoadState::Loading => {
                vec![Self::message(&id, "Loading tables...", rows, false)]
            }
            LoadState::Failed(error) => vec![Self::message(&id, error, rows, true)],
            LoadState::Loaded(tables) if tables.is_empty() => {
                vec![Self::message(&id, "No tables", rows, false)]
            }
            LoadState::Loaded(tables) => tables
                .iter()
                .map(|table| self.table_item(profile_id, &schema.name, table, rows))
                .collect(),
        };
        Self::add_row(
            rows,
            id,
            schema.name.clone(),
            RowInfo::folder(
                RowAction::Schema {
                    profile_id: profile_id.to_owned(),
                    name: schema.name.clone(),
                },
                Some("icons/schema.svg"),
                None,
            ),
            children,
            self.expanded_items.contains(&format!("schema:{key}")),
        )
    }

    fn table_item(
        &self,
        profile_id: &str,
        schema_name: &str,
        table: &DatabaseTable,
        rows: &mut HashMap<SharedString, RowInfo>,
    ) -> TreeItem {
        let key = Self::table_key(profile_id, schema_name, &table.name);
        let sections = vec![
            self.section_item(
                &key,
                TableSection::Columns,
                &table.columns,
                labels::column_label,
                rows,
            ),
            self.section_item(
                &key,
                TableSection::Constraints,
                &table.constraints,
                labels::constraint_label,
                rows,
            ),
            self.section_item(
                &key,
                TableSection::Indexes,
                &table.indexes,
                labels::index_label,
                rows,
            ),
        ];
        Self::add_row(
            rows,
            format!("table:{key}"),
            table.name.clone(),
            RowInfo::folder(
                RowAction::Table(TableTarget {
                    profile_id: profile_id.to_owned(),
                    schema_name: schema_name.to_owned(),
                    table_name: table.name.clone(),
                }),
                Some("icons/table.svg"),
                None,
            ),
            sections,
            self.expanded_items.contains(&format!("table:{key}")),
        )
    }

    fn section_item<T>(
        &self,
        table_key: &str,
        section: TableSection,
        items: &[T],
        label: fn(&T) -> String,
        rows: &mut HashMap<SharedString, RowInfo>,
    ) -> TreeItem {
        let key = Self::section_key(table_key, section);
        let id = format!("section:{key}");
        let children = if items.is_empty() {
            vec![Self::message(
                &id,
                &format!("No {}", section.name()),
                rows,
                false,
            )]
        } else {
            items
                .iter()
                .enumerate()
                .map(|(index, item)| {
                    Self::add_row(
                        rows,
                        format!("{id}:item:{index}"),
                        label(item),
                        RowInfo::leaf(Some(section.icon()), false),
                        Vec::new(),
                        false,
                    )
                })
                .collect()
        };
        Self::add_row(
            rows,
            id,
            format!("{} ({})", section.title(), items.len()),
            RowInfo::folder(RowAction::Section, None, None),
            children,
            self.expanded_items.contains(&format!("section:{key}")),
        )
    }
}

impl Render for DatabaseTree {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = cx.theme();
        let rows = self.rows.clone();
        let view = cx.entity().downgrade();
        let empty = self.profile_store.read(cx).profiles().is_empty();

        div()
            .size_full()
            .flex()
            .flex_col()
            .bg(theme.sidebar)
            .text_color(theme.sidebar_foreground)
            .child(
                div()
                    .h(px(44.))
                    .flex_none()
                    .flex()
                    .items_center()
                    .justify_between()
                    .px_3()
                    .border_b_1()
                    .border_color(theme.sidebar_border)
                    .child(
                        div()
                            .font_weight(gpui::FontWeight::SEMIBOLD)
                            .child("Database"),
                    )
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .gap_1()
                            .child(
                                Button::new("add-database-profile")
                                    .ghost()
                                    .small()
                                    .icon(IconName::Plus)
                                    .tooltip("Add database connection"),
                            )
                            .child(
                                Button::new("refresh-database-catalog")
                                    .ghost()
                                    .small()
                                    .icon(IconName::RefreshCw)
                                    .tooltip("Refresh database catalog"),
                            )
                            .child(
                                Button::new("database-actions")
                                    .ghost()
                                    .small()
                                    .icon(IconName::Ellipsis)
                                    .tooltip("More database actions"),
                            ),
                    ),
            )
            .child(
                div()
                    .flex_1()
                    .min_h_0()
                    .when(empty, |content| {
                        content.child(
                            div()
                                .p_2()
                                .text_sm()
                                .text_color(theme.muted_foreground)
                                .child("No database profiles. Use + to add one."),
                        )
                    })
                    .when(!empty, |content| {
                        content.child(tree(&self.tree_state, move |_, entry, selected, _, cx| {
                            let theme = cx.theme();
                            let row = rows.get(&entry.item().id);
                            let icon = row.and_then(|row| row.icon);
                            let status = row
                                .and_then(|row| row.status)
                                .map(|status| status.color(theme));
                            let color = if row.is_some_and(|row| row.error) {
                                theme.danger
                            } else {
                                theme.sidebar_foreground
                            };
                            let id = entry.item().id.clone();
                            let select_id = id.clone();
                            let select_view = view.clone();
                            let activate_view = view.clone();
                            let arrow_id = id.clone();
                            let arrow_select_id = id.clone();
                            let arrow_view = view.clone();
                            let arrow_select_view = view.clone();
                            ListItem::new(id.clone())
                                .on_mouse_down(MouseButton::Left, move |_, window, cx| {
                                    // The library tree toggles on mouse-down unless we intercept it.
                                    cx.stop_propagation();
                                    if let Some(view) = select_view.upgrade() {
                                        view.update(cx, |view, cx| {
                                            view.select_row(&select_id, window, cx)
                                        });
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
                                .selected(selected)
                                .h_7()
                                .pl(px(12. + entry.depth() as f32 * 16.))
                                .text_sm()
                                .text_color(color)
                                .child(
                                    div()
                                        .flex()
                                        .items_center()
                                        .w_full()
                                        .min_w_0()
                                        .child(
                                            div()
                                                .id("disclosure")
                                                .w_4()
                                                .flex_none()
                                                .when(entry.is_folder(), |arrow| {
                                                    arrow
                                                        .cursor_pointer()
                                                        .on_mouse_down(
                                                            MouseButton::Left,
                                                            move |_, window, cx| {
                                                                cx.stop_propagation();
                                                                if let Some(view) =
                                                                    arrow_select_view.upgrade()
                                                                {
                                                                    view.update(cx, |view, cx| {
                                                                        view.select_row(
                                                                            &arrow_select_id,
                                                                            window,
                                                                            cx,
                                                                        )
                                                                    });
                                                                }
                                                            },
                                                        )
                                                        .on_click(move |event, _, cx| {
                                                            cx.stop_propagation();
                                                            if let Some(view) = arrow_view.upgrade()
                                                            {
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
                                                .child(if entry.is_folder() {
                                                    if entry.is_expanded() { "▾" } else { "▸" }
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
                                                .child(entry.item().label.clone()),
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
                                        }),
                                )
                        }))
                    }),
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
                    .border_color(theme.sidebar_border)
                    .text_sm()
                    .text_color(theme.muted_foreground)
                    .child("●")
                    .child("PostgreSQL 16"),
            )
    }
}

#[cfg(test)]
mod tests {
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

    #[test]
    fn connection_status_colors_follow_the_current_palette() {
        let mut theme = Theme::default();
        assert_eq!(
            ConnectionStatus::Disconnected.color(&theme),
            theme.muted_foreground
        );
        assert_eq!(ConnectionStatus::Connecting.color(&theme), theme.warning);
        assert_eq!(ConnectionStatus::Failed.color(&theme), theme.danger);
        assert_eq!(ConnectionStatus::Connected.color(&theme), theme.success);

        let changed_color = gpui::hsla(0.4, 0.7, 0.6, 1.0);
        theme.colors.success = changed_color;
        assert_eq!(ConnectionStatus::Connected.color(&theme), changed_color);
    }

    #[test]
    fn table_keys_separate_schema_and_table_names() {
        assert_ne!(
            DatabaseTree::table_key("profile", "a:b", "c"),
            DatabaseTree::table_key("profile", "a", "b:c"),
        );
    }
}
