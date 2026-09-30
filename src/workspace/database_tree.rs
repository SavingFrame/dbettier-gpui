use std::collections::HashSet;

use gpui::{Context, Div, Entity, Rgba, Subscription, Window, div, prelude::*, px, svg};

use crate::{
    database::{
        ConstraintType, DatabaseConnectionProfile, DatabaseSchema, DatabaseTable, LoadState,
        TableColumn, TableConstraint, TableIndex, profile_store::DatabaseProfileStore,
        session::DatabaseSessionState, session_store::DatabaseSessionStore,
    },
    theme::AppTheme,
};

enum ProfileSession {
    Connecting,
    Connected(LoadState<Vec<DatabaseSchema>>),
    Failed,
}

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

pub(crate) struct DatabaseTree {
    profile_store: Entity<DatabaseProfileStore>,
    session_store: Entity<DatabaseSessionStore>,
    expanded_profiles: HashSet<String>,
    expanded_schemas: HashSet<String>,
    expanded_tables: HashSet<String>,
    expanded_sections: HashSet<String>,
    _profile_store_subscription: Subscription,
    _session_store_subscription: Subscription,
}

impl DatabaseTree {
    pub(crate) fn new(cx: &mut Context<Self>) -> Self {
        let profile_store = DatabaseProfileStore::global(cx);
        let session_store = DatabaseSessionStore::global(cx);
        let profile_store_subscription = cx.observe(&profile_store, |_, _, cx| cx.notify());
        let session_store_subscription = cx.observe(&session_store, |_, _, cx| cx.notify());

        Self {
            profile_store,
            session_store,
            expanded_profiles: HashSet::new(),
            expanded_schemas: HashSet::new(),
            expanded_tables: HashSet::new(),
            expanded_sections: HashSet::new(),
            _profile_store_subscription: profile_store_subscription,
            _session_store_subscription: session_store_subscription,
        }
    }

    fn render_row(
        label: &str,
        depth: f32,
        selected: bool,
        status_color: Option<Rgba>,
        disclosure: Option<bool>,
        icon_path: Option<&'static str>,
        theme: &AppTheme,
    ) -> Div {
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
            .child(
                div()
                    .w_4()
                    .flex_none()
                    .text_color(theme.text_muted)
                    .when_some(disclosure, |disclosure, expanded| {
                        disclosure.child(if expanded { "▾" } else { "▸" })
                    }),
            )
            .when_some(icon_path, |row, icon_path| {
                row.child(
                    svg()
                        .path(icon_path)
                        .size_4()
                        .flex_none()
                        .mr_2()
                        .text_color(theme.text_muted),
                )
            })
            .child(
                div()
                    .min_w_0()
                    .flex_1()
                    .overflow_hidden()
                    .whitespace_nowrap()
                    .text_ellipsis()
                    .child(label.to_owned()),
            )
            .when_some(status_color, |row, status_color| {
                row.child(
                    div()
                        .flex_none()
                        .w(px(7.))
                        .h(px(7.))
                        .rounded_full()
                        .bg(status_color),
                )
            });

        if selected {
            row.bg(theme.selection)
        } else {
            row
        }
    }

    fn message_row(label: &str, depth: f32, theme: &AppTheme) -> Div {
        Self::render_row(label, depth, false, None, None, None, theme)
    }

    fn loading_rows<T>(
        state: &LoadState<Vec<T>>,
        depth: f32,
        loading: &str,
        empty: &str,
        theme: &AppTheme,
        render_item: impl FnMut(&T) -> Div,
    ) -> Vec<Div> {
        match state {
            LoadState::NotLoaded | LoadState::Loading => {
                vec![Self::message_row(loading, depth, theme)]
            }
            LoadState::Loaded(items) if items.is_empty() => {
                vec![Self::message_row(empty, depth, theme)]
            }
            LoadState::Loaded(items) => items.iter().map(render_item).collect(),
            LoadState::Failed(error) => {
                vec![Self::message_row(error, depth, theme).text_color(theme.error)]
            }
        }
    }

    fn render_profile(
        &self,
        profile: &DatabaseConnectionProfile,
        theme: &AppTheme,
        cx: &mut Context<Self>,
    ) -> Div {
        let session_state = self
            .session_store
            .read(cx)
            .session(&profile.uuid)
            .map(|session| match session.read(cx).state() {
                DatabaseSessionState::Connecting => ProfileSession::Connecting,
                DatabaseSessionState::Connected { schemas, .. } => {
                    ProfileSession::Connected(schemas.clone())
                }
                DatabaseSessionState::Failed(_) => ProfileSession::Failed,
            });
        let status_color = match &session_state {
            None => theme.text_muted,
            Some(ProfileSession::Connecting) => theme.warning,
            Some(ProfileSession::Connected(_)) => theme.success,
            Some(ProfileSession::Failed) => theme.error,
        };
        let expanded = self.expanded_profiles.contains(&profile.uuid);
        let profile_for_click = profile.clone();
        let row = Self::render_row(
            &profile.name,
            0.,
            false,
            Some(status_color),
            Some(expanded),
            Some("icons/database.svg"),
            theme,
        )
        .id(format!("database-profile-{}", profile.uuid))
        .cursor_pointer()
        .on_click(cx.listener(move |tree, _, _, cx| {
            tree.toggle_profile(profile_for_click.clone(), cx);
        }));

        let children = if expanded {
            match session_state {
                None => vec![Self::message_row("Starting connection...", 1., theme)],
                Some(ProfileSession::Connecting) => {
                    vec![Self::message_row("Connecting...", 1., theme)]
                }
                Some(ProfileSession::Failed) => Vec::new(),
                Some(ProfileSession::Connected(schemas)) => Self::loading_rows(
                    &schemas,
                    1.,
                    "Loading schemas...",
                    "No schemas",
                    theme,
                    |schema| self.render_schema(&profile.uuid, schema, theme, cx),
                ),
            }
        } else {
            Vec::new()
        };

        div().flex().flex_col().child(row).children(children)
    }

    fn render_schema(
        &self,
        profile_id: &str,
        schema: &DatabaseSchema,
        theme: &AppTheme,
        cx: &mut Context<Self>,
    ) -> Div {
        let schema_key = Self::schema_key(profile_id, &schema.name);
        let expanded = self.expanded_schemas.contains(&schema_key);
        let profile_id_for_click = profile_id.to_owned();
        let schema_name_for_click = schema.name.clone();
        let row = Self::render_row(
            &schema.name,
            1.,
            false,
            None,
            Some(expanded),
            Some("icons/schema.svg"),
            theme,
        )
        .id(format!("database-schema-{profile_id}-{schema_key}"))
        .cursor_pointer()
        .on_click(cx.listener(move |tree, _, _, cx| {
            tree.toggle_schema(
                profile_id_for_click.clone(),
                schema_name_for_click.clone(),
                cx,
            );
        }));

        let children = if expanded {
            Self::loading_rows(
                &schema.tables,
                2.,
                "Loading tables...",
                "No tables",
                theme,
                |table| self.render_table(profile_id, &schema.name, table, theme, cx),
            )
        } else {
            Vec::new()
        };
        div().flex().flex_col().child(row).children(children)
    }

    fn render_table(
        &self,
        profile_id: &str,
        schema_name: &str,
        table: &DatabaseTable,
        theme: &AppTheme,
        cx: &mut Context<Self>,
    ) -> Div {
        let table_key = Self::table_key(profile_id, schema_name, &table.name);
        let expanded = self.expanded_tables.contains(&table_key);
        let key_for_click = table_key.clone();
        let row = Self::render_row(
            &table.name,
            2.,
            false,
            None,
            Some(expanded),
            Some("icons/table.svg"),
            theme,
        )
        .id(format!("database-table-{table_key}"))
        .cursor_pointer()
        .on_click(cx.listener(move |tree, _, _, cx| {
            tree.toggle_table(key_for_click.clone(), cx);
        }));

        let mut node = div().flex().flex_col().child(row);
        if expanded {
            node = node
                .child(self.render_section(
                    &table_key,
                    TableSection::Columns,
                    &table.columns,
                    Self::column_label,
                    theme,
                    cx,
                ))
                .child(self.render_section(
                    &table_key,
                    TableSection::Constraints,
                    &table.constraints,
                    Self::constraint_label,
                    theme,
                    cx,
                ))
                .child(self.render_section(
                    &table_key,
                    TableSection::Indexes,
                    &table.indexes,
                    Self::index_label,
                    theme,
                    cx,
                ));
        }
        node
    }

    fn render_section<T>(
        &self,
        table_key: &str,
        section: TableSection,
        items: &[T],
        label: fn(&T) -> String,
        theme: &AppTheme,
        cx: &mut Context<Self>,
    ) -> Div {
        let section_key = Self::section_key(table_key, section.name());
        let expanded = self.expanded_sections.contains(&section_key);
        let key_for_click = section_key.clone();
        let row = Self::render_row(
            &format!("{} ({})", section.title(), items.len()),
            3.,
            false,
            None,
            Some(expanded),
            None,
            theme,
        )
        .id(format!("database-section-{section_key}"))
        .cursor_pointer()
        .on_click(cx.listener(move |tree, _, _, cx| {
            tree.toggle_section(key_for_click.clone(), cx);
        }));

        div().flex().flex_col().child(row).when(expanded, |node| {
            node.children(items.iter().map(|item| {
                Self::render_row(
                    &label(item),
                    4.,
                    false,
                    None,
                    None,
                    Some(section.icon()),
                    theme,
                )
            }))
        })
    }

    fn toggle_profile(&mut self, profile: DatabaseConnectionProfile, cx: &mut Context<Self>) {
        if !self.expanded_profiles.insert(profile.uuid.clone()) {
            self.expanded_profiles.remove(&profile.uuid);
            cx.notify();
            return;
        }

        self.session_store
            .update(cx, |session_store, cx| session_store.connect(profile, cx));
        cx.notify();
    }

    fn schema_key(profile_id: &str, schema_name: &str) -> String {
        format!("{profile_id}:{schema_name}")
    }

    fn toggle_schema(&mut self, profile_id: String, schema_name: String, cx: &mut Context<Self>) {
        let schema_key = Self::schema_key(&profile_id, &schema_name);
        if !self.expanded_schemas.insert(schema_key.clone()) {
            self.expanded_schemas.remove(&schema_key);
            cx.notify();
            return;
        }

        self.session_store.update(cx, |session_store, cx| {
            session_store.load_tables(&profile_id, &schema_name, cx);
        });
        cx.notify();
    }

    fn table_key(profile_id: &str, schema_name: &str, table_name: &str) -> String {
        format!("{profile_id}:{schema_name}:{table_name}")
    }

    fn section_key(table_key: &str, section: &str) -> String {
        format!("{table_key}:{section}")
    }

    fn toggle_table(&mut self, table_key: String, cx: &mut Context<Self>) {
        if !self.expanded_tables.insert(table_key.clone()) {
            self.expanded_tables.remove(&table_key);
        } else {
            for section in [
                TableSection::Columns,
                TableSection::Constraints,
                TableSection::Indexes,
            ] {
                self.expanded_sections
                    .insert(Self::section_key(&table_key, section.name()));
            }
        }
        cx.notify();
    }

    fn toggle_section(&mut self, section_key: String, cx: &mut Context<Self>) {
        if !self.expanded_sections.insert(section_key.clone()) {
            self.expanded_sections.remove(&section_key);
        }
        cx.notify();
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
        let profiles = self.profile_store.read(cx).profiles().to_vec();
        let theme = cx.global::<AppTheme>().clone();

        div()
            .size_full()
            .flex()
            .flex_col()
            .bg(theme.panel)
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
                    .id("database-tree-content")
                    .flex_1()
                    .overflow_y_scroll()
                    .p_2()
                    .when(profiles.is_empty(), |tree| {
                        tree.child(
                            div()
                                .p_2()
                                .text_sm()
                                .text_color(theme.text_muted)
                                .child("No database profiles. Use + to add one."),
                        )
                    })
                    .children(
                        profiles
                            .iter()
                            .map(|profile| self.render_profile(profile, &theme, cx)),
                    ),
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
