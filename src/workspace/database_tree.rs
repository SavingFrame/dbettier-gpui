use std::collections::HashSet;

use gpui::{Context, Div, Entity, Rgba, Subscription, Window, div, prelude::*, px, svg};

use crate::{
    database::{
        DatabaseConnectionProfile, LoadState, profile_store::DatabaseProfileStore,
        session::DatabaseSessionState, session_store::DatabaseSessionStore,
    },
    theme::AppTheme,
};

pub(crate) struct DatabaseTree {
    profile_store: Entity<DatabaseProfileStore>,
    session_store: Entity<DatabaseSessionStore>,
    expanded_profiles: HashSet<String>,
    expanded_schemas: HashSet<String>,
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
}

impl Render for DatabaseTree {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let profile_store = self.profile_store.read(cx);
        let session_store = self.session_store.read(cx);
        let theme = cx.global::<AppTheme>();

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
                        let session_state = session_store
                            .session(&profile.uuid)
                            .map(|session| session.read(cx).state());
                        let status_color = match session_state {
                            None => theme.text_muted,
                            Some(DatabaseSessionState::Connecting) => theme.warning,
                            Some(DatabaseSessionState::Connected { .. }) => theme.success,
                            Some(DatabaseSessionState::Failed(_)) => theme.error,
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
                        .on_click(cx.listener(
                            move |database_tree, _, _, cx| {
                                database_tree.toggle_profile(profile_for_click.clone(), cx);
                            },
                        ));

                        let children = if expanded {
                            match session_state {
                                None => vec![Self::render_row(
                                    "Starting connection...",
                                    1.,
                                    false,
                                    None,
                                    None,
                                    None,
                                    theme,
                                )],
                                Some(DatabaseSessionState::Connecting) => vec![Self::render_row(
                                    "Connecting...",
                                    1.,
                                    false,
                                    None,
                                    None,
                                    None,
                                    theme,
                                )],
                                Some(DatabaseSessionState::Failed(_)) => Vec::new(),
                                Some(DatabaseSessionState::Connected { schemas, .. }) => {
                                    match schemas {
                                        LoadState::NotLoaded | LoadState::Loading => {
                                            vec![Self::render_row(
                                                "Loading schemas...",
                                                1.,
                                                false,
                                                None,
                                                None,
                                                None,
                                                theme,
                                            )]
                                        }
                                        LoadState::Loaded(schemas) if schemas.is_empty() => {
                                            vec![Self::render_row(
                                                "No schemas",
                                                1.,
                                                false,
                                                None,
                                                None,
                                                None,
                                                theme,
                                            )]
                                        }
                                        LoadState::Loaded(schemas) => schemas
                                            .iter()
                                            .map(|schema| {
                                                let schema_key =
                                                    Self::schema_key(&profile.uuid, &schema.name);
                                                let schema_expanded =
                                                    self.expanded_schemas.contains(&schema_key);
                                                let profile_id = profile.uuid.clone();
                                                let schema_name = schema.name.clone();
                                                let schema_row = Self::render_row(
                                                    &schema.name,
                                                    1.,
                                                    false,
                                                    None,
                                                    Some(schema_expanded),
                                                    Some("icons/schema.svg"),
                                                    theme,
                                                )
                                                .id(format!(
                                                    "database-schema-{}-{schema_key}",
                                                    profile.uuid
                                                ))
                                                .cursor_pointer()
                                                .on_click(cx.listener(
                                                    move |database_tree, _, _, cx| {
                                                        database_tree.toggle_schema(
                                                            profile_id.clone(),
                                                            schema_name.clone(),
                                                            cx,
                                                        );
                                                    },
                                                ));

                                                let tables = if schema_expanded {
                                                    match &schema.tables {
                                                        LoadState::NotLoaded
                                                        | LoadState::Loading => {
                                                            vec![Self::render_row(
                                                                "Loading tables...",
                                                                2.,
                                                                false,
                                                                None,
                                                                None,
                                                                None,
                                                                theme,
                                                            )]
                                                        }
                                                        LoadState::Loaded(tables)
                                                            if tables.is_empty() =>
                                                        {
                                                            vec![Self::render_row(
                                                                "No tables",
                                                                2.,
                                                                false,
                                                                None,
                                                                None,
                                                                None,
                                                                theme,
                                                            )]
                                                        }
                                                        LoadState::Loaded(tables) => tables
                                                            .iter()
                                                            .map(|table| {
                                                                Self::render_row(
                                                                    &table.name,
                                                                    2.,
                                                                    false,
                                                                    None,
                                                                    None,
                                                                    Some("icons/table.svg"),
                                                                    theme,
                                                                )
                                                            })
                                                            .collect(),
                                                        LoadState::Failed(error) => vec![
                                                            Self::render_row(
                                                                error, 2., false, None, None, None,
                                                                theme,
                                                            )
                                                            .text_color(theme.error),
                                                        ],
                                                    }
                                                } else {
                                                    Vec::new()
                                                };

                                                div()
                                                    .flex()
                                                    .flex_col()
                                                    .child(schema_row)
                                                    .children(tables)
                                            })
                                            .collect(),
                                        LoadState::Failed(error) => vec![
                                            Self::render_row(
                                                error, 1., false, None, None, None, theme,
                                            )
                                            .text_color(theme.error),
                                        ],
                                    }
                                }
                            }
                        } else {
                            Vec::new()
                        };

                        div().flex().flex_col().child(row).children(children)
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
