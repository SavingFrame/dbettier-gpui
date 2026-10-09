use std::collections::{HashMap, HashSet};

use gpui::{Hsla, SharedString};
use gpui_kit::{
    assets::IconName,
    component::{Theme, tree::TreeItem},
};

use crate::database::{
    DatabaseConnectionProfile, DatabaseSchema, DatabaseTable, LoadState,
    session::DatabaseSessionState,
};

use super::labels;

#[derive(Clone, Copy)]
pub(super) enum TableSection {
    Columns,
    Constraints,
    Indexes,
}

impl TableSection {
    pub(super) fn name(self) -> &'static str {
        match self {
            Self::Columns => "columns",
            Self::Constraints => "constraints",
            Self::Indexes => "indexes",
        }
    }

    pub(super) fn title(self) -> &'static str {
        match self {
            Self::Columns => "Columns",
            Self::Constraints => "Constraints",
            Self::Indexes => "Indexes",
        }
    }

    pub(super) fn icon(self) -> IconName {
        match self {
            Self::Columns => IconName::Columns3,
            Self::Constraints => IconName::KeyRound,
            Self::Indexes => IconName::ListOrdered,
        }
    }
}

#[derive(Clone)]
pub(super) enum RowAction {
    Profile(DatabaseConnectionProfile),
    Schema { profile_id: String, name: String },
    Table(TableTarget),
    Section,
}

#[derive(Clone)]
pub(crate) struct TableTarget {
    pub(crate) profile_id: String,
    pub(crate) schema_name: String,
    pub(crate) table_name: String,
}

#[derive(Clone)]
pub(crate) struct SqlTarget {
    pub(crate) profile_id: String,
    pub(crate) schema_name: Option<String>,
}

#[derive(Clone)]
pub(crate) enum Target {
    Table(TableTarget),
    Sql(SqlTarget),
}

impl Target {
    pub(crate) fn profile_id(&self) -> String {
        match self {
            Target::Table(table_target) => table_target.profile_id.clone(),
            Target::Sql(sql_target) => sql_target.profile_id.clone(),
        }
    }
}

#[derive(Clone, Copy)]
pub(super) enum ConnectionStatus {
    Disconnected,
    Connecting,
    Failed,
    Connected,
}

impl ConnectionStatus {
    pub(super) fn color(self, theme: &Theme) -> Hsla {
        match self {
            Self::Disconnected => theme.muted_foreground,
            Self::Connecting => theme.warning,
            Self::Failed => theme.danger,
            Self::Connected => theme.success,
        }
    }
}

pub(super) struct RowInfo {
    pub(super) action: Option<RowAction>,
    pub(super) icon: Option<IconName>,
    pub(super) status: Option<ConnectionStatus>,
    pub(super) error: bool,
}

impl RowInfo {
    pub(super) fn folder(
        action: RowAction,
        icon: Option<IconName>,
        status: Option<ConnectionStatus>,
    ) -> Self {
        Self {
            action: Some(action),
            icon,
            status,
            error: false,
        }
    }

    pub(super) fn leaf(icon: Option<IconName>, error: bool) -> Self {
        Self {
            action: None,
            icon,
            status: None,
            error,
        }
    }
}

pub(super) fn schema_key(profile_id: &str, schema_name: &str) -> String {
    format!("{profile_id}:{}:{schema_name}", schema_name.len())
}

pub(super) fn table_key(profile_id: &str, schema_name: &str, table_name: &str) -> String {
    format!(
        "{}:{}:{table_name}",
        schema_key(profile_id, schema_name),
        table_name.len()
    )
}

pub(super) fn section_key(table_key: &str, section: TableSection) -> String {
    format!("{table_key}:{}", section.name())
}

pub(super) struct TreeBuilder<'a> {
    rows: HashMap<SharedString, RowInfo>,
    expanded_items: &'a HashSet<String>,
}

impl<'a> TreeBuilder<'a> {
    pub(super) fn new(expanded_items: &'a HashSet<String>) -> Self {
        Self {
            rows: HashMap::new(),
            expanded_items,
        }
    }

    pub(super) fn finish(self) -> HashMap<SharedString, RowInfo> {
        self.rows
    }

    fn add_row(
        &mut self,
        id: String,
        label: impl Into<SharedString>,
        info: RowInfo,
        children: Vec<TreeItem>,
        expanded: bool,
    ) -> TreeItem {
        let id: SharedString = id.into();
        self.rows.insert(id.clone(), info);
        TreeItem::new(id, label)
            .children(children)
            .expanded(expanded)
    }

    fn message(&mut self, id: &str, label: &str, error: bool) -> TreeItem {
        let id: SharedString = format!("{id}:message").into();
        self.rows.insert(id.clone(), RowInfo::leaf(None, error));
        TreeItem::new(id, label).disabled(true)
    }

    pub(super) fn profile_item(
        &mut self,
        profile: &DatabaseConnectionProfile,
        session: Option<&DatabaseSessionState>,
    ) -> TreeItem {
        let id = format!("profile:{}", profile.uuid);
        let (status, children) = match session {
            None | Some(DatabaseSessionState::Disconnected) => (
                ConnectionStatus::Disconnected,
                vec![self.message(&id, "Starting connection...", false)],
            ),
            Some(DatabaseSessionState::Connecting) => (
                ConnectionStatus::Connecting,
                vec![self.message(&id, "Connecting...", false)],
            ),
            Some(DatabaseSessionState::Failed(error)) => (
                ConnectionStatus::Failed,
                vec![self.message(&id, error, true)],
            ),
            Some(DatabaseSessionState::Connected { schemas, .. }) => {
                let children = match schemas {
                    LoadState::NotLoaded | LoadState::Loading => {
                        vec![self.message(&id, "Loading schemas...", false)]
                    }
                    LoadState::Failed(error) => vec![self.message(&id, error, true)],
                    LoadState::Loaded(schemas) if schemas.is_empty() => {
                        vec![self.message(&id, "No schemas", false)]
                    }
                    LoadState::Loaded(schemas) => schemas
                        .iter()
                        .map(|schema| self.schema_item(&profile.uuid, schema))
                        .collect(),
                };
                (ConnectionStatus::Connected, children)
            }
        };
        self.add_row(
            id,
            profile.name.clone(),
            RowInfo::folder(
                RowAction::Profile(profile.clone()),
                Some(IconName::Database),
                Some(status),
            ),
            children,
            self.expanded_items
                .contains(&format!("profile:{}", profile.uuid)),
        )
    }

    fn schema_item(&mut self, profile_id: &str, schema: &DatabaseSchema) -> TreeItem {
        let key = schema_key(profile_id, &schema.name);
        let id = format!("schema:{key}");
        let children = match &schema.tables {
            LoadState::NotLoaded | LoadState::Loading => {
                vec![self.message(&id, "Loading tables...", false)]
            }
            LoadState::Failed(error) => vec![self.message(&id, error, true)],
            LoadState::Loaded(tables) if tables.is_empty() => {
                vec![self.message(&id, "No tables", false)]
            }
            LoadState::Loaded(tables) => tables
                .iter()
                .map(|table| self.table_item(profile_id, &schema.name, table))
                .collect(),
        };
        self.add_row(
            id,
            schema.name.clone(),
            RowInfo::folder(
                RowAction::Schema {
                    profile_id: profile_id.to_owned(),
                    name: schema.name.clone(),
                },
                Some(IconName::Network),
                None,
            ),
            children,
            self.expanded_items.contains(&format!("schema:{key}")),
        )
    }

    fn table_item(
        &mut self,
        profile_id: &str,
        schema_name: &str,
        table: &DatabaseTable,
    ) -> TreeItem {
        let key = table_key(profile_id, schema_name, &table.name);
        let sections = vec![
            self.section_item(
                &key,
                TableSection::Columns,
                &table.columns,
                labels::column_label,
            ),
            self.section_item(
                &key,
                TableSection::Constraints,
                &table.constraints,
                labels::constraint_label,
            ),
            self.section_item(
                &key,
                TableSection::Indexes,
                &table.indexes,
                labels::index_label,
            ),
        ];
        self.add_row(
            format!("table:{key}"),
            table.name.clone(),
            RowInfo::folder(
                RowAction::Table(TableTarget {
                    profile_id: profile_id.to_owned(),
                    schema_name: schema_name.to_owned(),
                    table_name: table.name.clone(),
                }),
                Some(IconName::Table2),
                None,
            ),
            sections,
            self.expanded_items.contains(&format!("table:{key}")),
        )
    }

    fn section_item<T>(
        &mut self,
        table_key: &str,
        section: TableSection,
        items: &[T],
        label: fn(&T) -> String,
    ) -> TreeItem {
        let key = section_key(table_key, section);
        let id = format!("section:{key}");
        let children = if items.is_empty() {
            vec![self.message(&id, &format!("No {}", section.name()), false)]
        } else {
            items
                .iter()
                .enumerate()
                .map(|(index, item)| {
                    self.add_row(
                        format!("{id}:item:{index}"),
                        label(item),
                        RowInfo::leaf(Some(section.icon()), false),
                        Vec::new(),
                        false,
                    )
                })
                .collect()
        };
        self.add_row(
            id,
            format!("{} ({})", section.title(), items.len()),
            RowInfo::folder(RowAction::Section, None, None),
            children,
            self.expanded_items.contains(&format!("section:{key}")),
        )
    }
}
