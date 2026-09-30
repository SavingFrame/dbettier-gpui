use std::{
    collections::{HashMap, HashSet},
    rc::Rc,
};

use gpui::{Context, Entity, Hsla, SharedString, Subscription, Window, div, prelude::*, px, svg};
use gpui_kit::assets::IconName;
use gpui_kit::component::{
    ActiveTheme as _, Sizable as _, Theme,
    button::{Button, ButtonVariants as _},
    list::ListItem,
    tree::{TreeEvent, TreeItem, TreeState, tree},
};

use crate::database::{
    ConstraintType, DatabaseConnectionProfile, DatabaseSchema, DatabaseTable, LoadState,
    TableColumn, TableConstraint, TableIndex, profile_store::DatabaseProfileStore,
    session::DatabaseSessionState, session_store::DatabaseSessionStore,
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
    Table(String),
    Section(String),
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
    expanded_profiles: HashSet<String>,
    expanded_schemas: HashSet<String>,
    expanded_tables: HashSet<String>,
    expanded_sections: HashSet<String>,
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
            tree.schedule_rebuild(None, cx);
        });
        let session_store_subscription = cx.observe(&session_store, |tree, _, cx| {
            tree.schedule_rebuild(None, cx);
        });
        let tree_subscription = cx.subscribe(&tree_state, |tree, _, event: &TreeEvent, cx| {
            tree.on_tree_event(event, cx);
        });

        let mut tree = Self {
            profile_store,
            session_store,
            tree_state,
            rows: Rc::new(HashMap::new()),
            expanded_profiles: HashSet::new(),
            expanded_schemas: HashSet::new(),
            expanded_tables: HashSet::new(),
            expanded_sections: HashSet::new(),
            _profile_store_subscription: profile_store_subscription,
            _session_store_subscription: session_store_subscription,
            _tree_subscription: tree_subscription,
        };
        tree.rebuild(None, cx);
        tree
    }

    fn schedule_rebuild(&self, selected: Option<SharedString>, cx: &mut Context<Self>) {
        let entity = cx.entity();
        cx.defer(move |cx| {
            entity.update(cx, |tree, cx| tree.rebuild(selected, cx));
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

        match action {
            RowAction::Profile(profile) => {
                Self::set_expanded(&mut self.expanded_profiles, profile.uuid.clone(), expanded);
                if expanded {
                    self.session_store
                        .update(cx, |store, cx| store.connect(profile, cx));
                }
            }
            RowAction::Schema { profile_id, name } => {
                Self::set_expanded(
                    &mut self.expanded_schemas,
                    Self::schema_key(&profile_id, &name),
                    expanded,
                );
                if expanded {
                    self.session_store.update(cx, |store, cx| {
                        store.load_tables(&profile_id, &name, cx);
                    });
                }
            }
            RowAction::Table(key) => {
                Self::set_expanded(&mut self.expanded_tables, key.clone(), expanded);
                if expanded {
                    for section in [
                        TableSection::Columns,
                        TableSection::Constraints,
                        TableSection::Indexes,
                    ] {
                        self.expanded_sections
                            .insert(Self::section_key(&key, section));
                    }
                }
            }
            RowAction::Section(key) => {
                Self::set_expanded(&mut self.expanded_sections, key, expanded);
            }
        }
        self.schedule_rebuild(Some(id.clone()), cx);
    }

    fn set_expanded(expanded_items: &mut HashSet<String>, key: String, expanded: bool) {
        if expanded {
            expanded_items.insert(key);
        } else {
            expanded_items.remove(&key);
        }
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

    fn rebuild(&mut self, selected: Option<SharedString>, cx: &mut Context<Self>) {
        let profiles = self.profile_store.read(cx).profiles().to_vec();
        let mut rows = HashMap::new();
        let items: Vec<TreeItem> = profiles
            .iter()
            .map(|profile| self.profile_item(profile, &mut rows, cx))
            .collect();
        self.rows = Rc::new(rows);
        self.tree_state.update(cx, |state, cx| {
            let selected = selected.or_else(|| state.selected_item().map(|item| item.id.clone()));
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
            None => (
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
            self.expanded_profiles.contains(&profile.uuid),
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
            self.expanded_schemas.contains(&key),
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
                Self::column_label,
                rows,
            ),
            self.section_item(
                &key,
                TableSection::Constraints,
                &table.constraints,
                Self::constraint_label,
                rows,
            ),
            self.section_item(
                &key,
                TableSection::Indexes,
                &table.indexes,
                Self::index_label,
                rows,
            ),
        ];
        Self::add_row(
            rows,
            format!("table:{key}"),
            table.name.clone(),
            RowInfo::folder(RowAction::Table(key.clone()), Some("icons/table.svg"), None),
            sections,
            self.expanded_tables.contains(&key),
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
            RowInfo::folder(RowAction::Section(key.clone()), None, None),
            children,
            self.expanded_sections.contains(&key),
        )
    }

    fn column_label(column: &TableColumn) -> String {
        let mut data_type = column.data_type.clone();
        if let Some(maximum_length) = column.character_maximum_length {
            data_type.push_str(&format!("({maximum_length})"));
        }
        let mut attributes = Vec::new();
        if !column.is_nullable {
            attributes.push("not null".to_string());
        }
        if column.is_auto_increment {
            attributes.push("auto increment".to_string());
        }
        if let Some(default) = &column.default {
            attributes.push(format!("default {default}"));
        }
        let suffix = if attributes.is_empty() {
            String::new()
        } else {
            format!(" ({})", attributes.join(", "))
        };
        format!("{}: {data_type}{suffix}", column.name)
    }

    fn constraint_label(constraint: &TableConstraint) -> String {
        let constraint_type = match &constraint.constraint_type {
            ConstraintType::ForeignKey => "foreign key",
            ConstraintType::Unique => "unique",
            ConstraintType::PrimaryKey => "primary key",
            ConstraintType::Check => "check",
            ConstraintType::Other(value) => value,
        };
        let mut attributes = vec![constraint_type.to_string()];
        if constraint.is_deferrable {
            attributes.push("deferrable".to_string());
        }
        if constraint.nulls_distinct == Some(false) {
            attributes.push("nulls not distinct".to_string());
        }
        format!("{} ({})", constraint.name, attributes.join(", "))
    }

    fn index_label(index: &TableIndex) -> String {
        let mut attributes = Vec::new();
        if index.is_primary {
            attributes.push("primary");
        } else if index.is_unique {
            attributes.push("unique");
        }
        if attributes.is_empty() {
            format!("{}: {}", index.name, index.definition)
        } else {
            format!(
                "{} ({}): {}",
                index.name,
                attributes.join(", "),
                index.definition
            )
        }
    }
}

impl Render for DatabaseTree {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = cx.theme();
        let rows = self.rows.clone();
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
                        content.child(tree(
                            &self.tree_state,
                            move |index, entry, selected, _, cx| {
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
                                ListItem::new(index)
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
                                                    .w_4()
                                                    .flex_none()
                                                    .text_color(theme.muted_foreground)
                                                    .child(if entry.is_folder() {
                                                        if entry.is_expanded() {
                                                            "▾"
                                                        } else {
                                                            "▸"
                                                        }
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
                            },
                        ))
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

    #[test]
    fn column_labels_preserve_type_and_attributes() {
        let column = TableColumn {
            name: "name".to_owned(),
            default: Some("'anonymous'".to_owned()),
            is_nullable: false,
            data_type: "varchar".to_owned(),
            character_maximum_length: Some(80),
            is_auto_increment: false,
        };
        assert_eq!(
            DatabaseTree::column_label(&column),
            "name: varchar(80) (not null, default 'anonymous')"
        );
    }
}
