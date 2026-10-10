use gpui::{Context, Entity, Task};

use crate::{
    database::{
        result::{CellValue, QueryOutput},
        session::DatabaseSession,
    },
    workspace::{TableTarget, database_tree::SqlTarget},
};

#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum SortDirection {
    Ascending,
    Descending,
}

impl SortDirection {
    fn to_sql(self) -> &'static str {
        match self {
            Self::Ascending => "ASC",
            Self::Descending => "DESC",
        }
    }
}

#[derive(Clone, PartialEq, Eq)]
pub(crate) struct SortField {
    pub(crate) column_name: String,
    pub(crate) direction: SortDirection,
}

impl SortField {
    fn to_sql(&self) -> String {
        format!("{} {}", self.column_name, self.direction.to_sql())
    }
}

pub(crate) struct TableQuery {
    target: TableTarget,
    limit: Option<u64>,
    offset: u64,
    ordering: Vec<SortField>,
    filtering: Option<String>,
}

impl TableQuery {
    pub(crate) fn new(target: TableTarget) -> Self {
        Self {
            target,
            limit: Some(500),
            offset: 0,
            ordering: Vec::new(),
            filtering: None,
        }
    }

    pub(crate) fn set_limit(&mut self, limit: Option<u64>) -> &mut Self {
        self.limit = limit;
        self
    }

    pub(crate) fn set_offset(&mut self, offset: u64) -> &mut Self {
        self.offset = offset;
        self
    }

    pub(crate) fn offset(&self) -> u64 {
        self.offset
    }

    pub(crate) fn limit(&self) -> Option<u64> {
        self.limit
    }

    fn sql(&self) -> String {
        let table_name = self.target.table_name.clone();
        let schema_name = self.target.schema_name.clone();
        let from_clause = format!("\"{schema_name}\".\"{table_name}\"",);
        let offset = self.offset;
        let mut sql = format!("SELECT * from {from_clause}");
        if let Some(filtration) = self.filtering.clone() {
            sql.push_str(&format!(" WHERE {}", filtration));
        }
        if !self.ordering.is_empty() {
            sql.push_str(&self.build_order_query());
        }

        if let Some(limit) = self.limit() {
            let limit = limit + 1;
            sql.push_str(&format!(" LIMIT {limit}"));
        }
        sql.push_str(&format!(" OFFSET {offset}"));
        sql
    }

    fn total_rows_sql(&self) -> String {
        let table_name = self.target.table_name.clone();
        let schema_name = self.target.schema_name.clone();
        let from_clause = format!("\"{schema_name}\".\"{table_name}\"",);
        format!("SELECT COUNT(1) from {from_clause}")
    }

    fn build_order_query(&self) -> String {
        format!(" ORDER BY {}", self.ordering_sql())
    }

    pub(crate) fn ordering_sql(&self) -> String {
        self.ordering
            .iter()
            .map(|clause| clause.to_sql())
            .collect::<Vec<String>>()
            .join(", ")
    }

    pub(crate) fn set_filtering(&mut self, filtering: Option<String>) {
        self.filtering = filtering;
    }

    pub(crate) fn add_ordering(&mut self, sort_field: SortField) {
        if let Some(existing) = self
            .ordering
            .iter_mut()
            .find(|value| value.column_name == sort_field.column_name)
        {
            *existing = sort_field;
        } else {
            self.ordering.push(sort_field);
        }
    }

    pub(crate) fn remove_ordering(&mut self, column_name: String) {
        if let Some(index) = self
            .ordering
            .iter()
            .position(|value| value.column_name == column_name)
        {
            self.ordering.remove(index);
        }
    }
    pub(crate) fn ordering(&self) -> Vec<SortField> {
        self.ordering.clone()
    }
}

pub(crate) struct SqlQuery {
    sql: String,
    target: SqlTarget,
}

pub(crate) enum QuerySource {
    Sql(SqlQuery),
    // Other things like filters, pagination, etc
    Table(TableQuery),
}

impl QuerySource {
    pub(crate) fn sql(&self) -> String {
        match self {
            Self::Sql(sql_query) => sql_query.sql.clone(),
            Self::Table(table_query) => table_query.sql(),
        }
    }
}

pub(crate) enum QueryStatus {
    Idle,
    Running,
    Succeeded,
    Failed,
}

pub(crate) struct QueryState {
    pub(crate) query: QuerySource,
    pub(crate) results: Option<QueryOutput>,
    pub(crate) has_next: Option<bool>,
    pub(crate) total_rows: Option<u64>,
    pub(crate) error_string: Option<String>,
    pub(crate) query_task: Option<Task<()>>,
    pub(crate) total_rows_task: Option<Task<()>>,
    pub(crate) status: QueryStatus,
}

impl QueryState {
    pub(crate) fn from_table_target(target: TableTarget) -> Self {
        let query = QuerySource::Table(TableQuery::new(target));
        Self {
            query,
            status: QueryStatus::Idle,
            has_next: None,
            total_rows: None,
            total_rows_task: None,
            results: None,
            error_string: None,
            query_task: None,
        }
    }

    pub(crate) fn from_sql_target(sql: String, target: SqlTarget) -> Self {
        let query = QuerySource::Sql(SqlQuery { sql, target });
        Self {
            query,
            status: QueryStatus::Idle,
            results: None,
            total_rows: None,
            total_rows_task: None,
            has_next: None,
            error_string: None,
            query_task: None,
        }
    }

    pub(crate) fn set_sql(&mut self, sql: String) {
        let target = match &self.query {
            QuerySource::Sql(query) => query.target.clone(),
            QuerySource::Table(query) => SqlTarget {
                profile_id: query.target.profile_id.clone(),
                schema_name: Some(query.target.schema_name.clone()),
            },
        };
        self.query = QuerySource::Sql(SqlQuery { sql, target });
    }

    pub(crate) fn page_size(&self) -> u64 {
        match &self.query {
            QuerySource::Table(query) => query
                .limit()
                .unwrap_or_else(|| self.total_rows.unwrap_or(0)),
            QuerySource::Sql(_) => self
                .results
                .as_ref()
                .map_or(0, |results| results.rows.len() as u64),
        }
    }

    pub(crate) fn offset(&self) -> u64 {
        match &self.query {
            QuerySource::Table(query) => query.offset(),
            QuerySource::Sql(_) => 0,
        }
    }

    pub(crate) fn set_page_size(
        &mut self,
        cx: &mut Context<Self>,
        page_size: Option<u64>,
        session: Entity<DatabaseSession>,
    ) {
        if self.is_loading() {
            return;
        }
        let QuerySource::Table(query) = &mut self.query else {
            return;
        };
        query.set_limit(page_size);
        self.execute(cx, session);
    }

    pub(crate) fn has_next_page(&self) -> bool {
        self.has_next == Some(true)
    }

    pub(crate) fn has_previous_page(&self) -> bool {
        self.offset() > 0
    }

    pub(crate) fn next_page(&mut self, cx: &mut Context<Self>, session: Entity<DatabaseSession>) {
        if self.is_loading() || !self.has_next_page() {
            return;
        }
        let QuerySource::Table(query) = &mut self.query else {
            return;
        };

        let Some(limit) = query.limit() else {
            return;
        };

        let Some(next_offset) = query.offset().checked_add(limit) else {
            self.error_string = Some("Cannot advance page: offset overflow".to_owned());
            cx.notify();
            return;
        };
        query.set_offset(next_offset);
        self.execute(cx, session);
    }

    pub(crate) fn previous_page(
        &mut self,
        cx: &mut Context<Self>,
        session: Entity<DatabaseSession>,
    ) {
        if self.is_loading() || !self.has_previous_page() {
            return;
        }
        let QuerySource::Table(query) = &mut self.query else {
            return;
        };
        let Some(limit) = query.limit() else {
            return;
        };
        let next_offset = query.offset().saturating_sub(limit);
        query.set_offset(next_offset);
        self.execute(cx, session);
    }

    pub(crate) fn is_loading(&self) -> bool {
        matches!(self.status, QueryStatus::Idle | QueryStatus::Running)
    }

    pub(crate) fn first_page(&mut self, cx: &mut Context<Self>, session: Entity<DatabaseSession>) {
        if self.is_loading() || !self.has_previous_page() {
            return;
        }
        let QuerySource::Table(query) = &mut self.query else {
            return;
        };
        query.set_offset(0);
        self.execute(cx, session);
    }

    pub(crate) fn last_page(&mut self, cx: &mut Context<Self>, session: Entity<DatabaseSession>) {
        if self.is_loading() || !self.has_next_page() {
            return;
        }
        if self.total_rows.is_none() {
            self.fetch_total_rows(cx, session);
            return;
        }
        let QuerySource::Table(query) = &mut self.query else {
            return;
        };
        let Some(limit) = query.limit() else {
            return;
        };
        if limit == 0 {
            return;
        }
        if let Some(value) = self.total_rows {
            let offset = value.saturating_sub(1) / limit * limit;
            query.set_offset(offset);
            self.execute(cx, session);
        }
    }

    pub(crate) fn fetch_total_rows(
        &mut self,
        cx: &mut Context<Self>,
        session: Entity<DatabaseSession>,
    ) {
        match &self.query {
            QuerySource::Sql(_) => {
                self.total_rows = Some(
                    self.results
                        .as_ref()
                        .map_or(0, |results| results.rows.len() as u64),
                )
            }
            QuerySource::Table(table_query) => {
                self.total_rows_task = None;
                self.total_rows = None;
                let sql = table_query.total_rows_sql();
                let task = session.update(cx, |session, cx| session.execute_query(cx, sql));
                match task {
                    Ok(task) => {
                        self.total_rows_task = Some(cx.spawn(async move |state, cx| {
                            let result = task.await;
                            if let Err(error) = state.update(cx, |state, cx| {
                                match result {
                                    Ok(output) => {
                                        let count = output.rows.first().and_then(|row| row.first());
                                        match count {
                                            Some(CellValue::Integer(value)) => {
                                                match usize::try_from(*value) {
                                                    Ok(total) => {
                                                        state.total_rows = Some(total as u64)
                                                    }
                                                    Err(error) => {
                                                        state.error_string = Some(format!(
                                                            "Invalid row count: {error}"
                                                        ));
                                                    }
                                                }
                                            }
                                            _ => {
                                                state.error_string = Some(
                                                    "Count query returned non integer value"
                                                        .to_owned(),
                                                )
                                            }
                                        }
                                    }
                                    Err(error) => {
                                        eprintln!("failed to execute query: {error}");
                                    }
                                }
                                cx.notify();
                            }) {
                                eprintln!("failed to update query state: {error}");
                            }
                        }));
                    }
                    Err(error) => {
                        eprintln!("failed to do something: {error}");
                    }
                }
                cx.notify();
            }
        };
    }

    pub(crate) fn execute(&mut self, cx: &mut Context<Self>, session: Entity<DatabaseSession>) {
        self.query_task = None;
        self.error_string = None;
        self.status = QueryStatus::Running;
        let sql = self.query.sql();
        let task = session.update(cx, |session, cx| session.execute_query(cx, sql));
        match task {
            Ok(task) => {
                self.query_task = Some(cx.spawn(async move |state, cx| {
                    let result = task.await;
                    if let Err(error) = state.update(cx, |state, cx| {
                        match result {
                            Ok(mut output) => {
                                state.has_next = match &state.query {
                                    QuerySource::Table(table_query) => match table_query.limit() {
                                        Some(limit) => {
                                            let Ok(limit) = usize::try_from(limit) else {
                                                state.error_string = Some(
                                                    "Page size exceeds platform supported size"
                                                        .to_owned(),
                                                );
                                                state.status = QueryStatus::Failed;
                                                cx.notify();
                                                return;
                                            };
                                            let has_next = output.rows.len() > limit;
                                            output.rows.truncate(limit);
                                            Some(has_next)
                                        }
                                        None => Some(false),
                                    },
                                    QuerySource::Sql(_) => Some(false),
                                };
                                if matches!(&state.query, QuerySource::Sql(_)) {
                                    if let Ok(count) = u64::try_from(output.rows.len()) {
                                        state.total_rows = Some(count)
                                    }
                                }
                                state.results = Some(output);
                                state.status = QueryStatus::Succeeded;
                            }
                            Err(error) => {
                                eprintln!("failed to execute query: {error}");
                                state.error_string = Some(error);
                                state.status = QueryStatus::Failed;
                            }
                        }
                        cx.notify();
                    }) {
                        eprintln!("failed to update query state: {error}");
                    }
                }));
            }
            Err(error) => {
                self.error_string = Some(error);
                self.status = QueryStatus::Failed;
            }
        }
        cx.notify();
    }
}
