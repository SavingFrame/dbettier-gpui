mod labels;
mod model;
mod row;

use std::{
    collections::{HashMap, HashSet},
    rc::Rc,
};

use gpui::{
    Context, Entity, EventEmitter, SharedString, Subscription, Window, div, prelude::*, px,
};
use gpui_kit::assets::IconName;
use gpui_kit::component::{
    ActiveTheme as _, Sizable as _, Theme,
    button::{Button, ButtonVariants as _},
    dock::{BasePanel, Panel, PanelEvent},
    tree::{TreeEvent, TreeItem, TreeState, tree},
};

use crate::database::{profile_store::DatabaseProfileStore, session_store::DatabaseSessionStore};
use model::{RowAction, RowInfo, TableSection, TreeBuilder};
use row::{DatabaseTreeRow, RowInteraction};

pub(super) use model::TableTarget;

pub(crate) struct DatabaseTree {
    focus_handle: gpui::FocusHandle,
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

pub(super) enum DatabaseTreeEvent {
    OpenTable(TableTarget),
}

impl EventEmitter<DatabaseTreeEvent> for DatabaseTree {}

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
            focus_handle: cx.focus_handle(),
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
                    model::table_key(&target.profile_id, &target.schema_name, &target.table_name);
                for section in [
                    TableSection::Columns,
                    TableSection::Constraints,
                    TableSection::Indexes,
                ] {
                    self.expanded_items
                        .insert(format!("section:{}", model::section_key(&key, section)));
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
                    self.on_open_table(&target, cx);
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

    fn on_open_table(&mut self, target: &TableTarget, cx: &mut Context<Self>) {
        eprintln!(
            "Open table: profile={}, schema={}, table={}",
            target.profile_id, target.schema_name, target.table_name,
        );
        cx.emit(DatabaseTreeEvent::OpenTable(target.clone()));
    }

    fn rebuild(&mut self, cx: &mut Context<Self>) {
        let profiles = self.profile_store.read(cx).profiles().to_vec();
        let mut builder = TreeBuilder::new(&self.expanded_items);
        let items: Vec<TreeItem> = profiles
            .iter()
            .map(|profile| {
                let session = self.session_store.read(cx).session(&profile.uuid).cloned();
                builder.profile_item(
                    profile,
                    session.as_ref().map(|session| session.read(cx).state()),
                )
            })
            .collect();
        self.rows = Rc::new(builder.finish());
        self.tree_state.update(cx, |state, cx| {
            let selected = state.selected_item().map(|item| item.id.clone());
            state.set_items(items, cx);
            if let Some(id) = selected {
                state.set_selected_index(state.index_of(&id), cx);
            }
        });
        cx.notify();
    }
}

impl gpui::Focusable for DatabaseTree {
    fn focus_handle(&self, _: &gpui::App) -> gpui::FocusHandle {
        self.focus_handle.clone()
    }
}

impl gpui::EventEmitter<PanelEvent> for DatabaseTree {}

impl BasePanel for DatabaseTree {
    fn panel_name(&self) -> &'static str {
        "DatabaseTree"
    }

    fn closable(&self, _: &gpui::App) -> bool {
        false
    }
}

impl Panel for DatabaseTree {
    fn title(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        "Database"
    }

    fn inner_padding(&self, _: &gpui::App) -> bool {
        false
    }
}

impl Render for DatabaseTree {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = cx.theme();
        let rows = self.rows.clone();
        let view = cx.entity().downgrade();
        let empty = self.profile_store.read(cx).profiles().is_empty();

        div()
            .track_focus(&self.focus_handle)
            .size_full()
            .flex()
            .flex_col()
            .bg(theme.sidebar)
            .text_color(theme.sidebar_foreground)
            .child(render_header(theme))
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
                            DatabaseTreeRow::new(
                                entry,
                                selected,
                                rows.get(&entry.item().id),
                                view.clone(),
                            )
                            .render(cx)
                        }))
                    }),
            )
            .child(render_footer(theme))
    }
}

fn render_header(theme: &Theme) -> impl IntoElement {
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
        )
}

fn render_footer(theme: &Theme) -> impl IntoElement {
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
        .child("PostgreSQL 16")
}
