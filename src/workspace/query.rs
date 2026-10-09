use gpui::{Context, Entity, Task};

use crate::{
    database::{result::QueryOutput, session::DatabaseSession},
    workspace::{TableTarget, database_tree::SqlTarget},
};

pub(crate) struct TableQuery {
    target: TableTarget,
    limit: u32,
    offset: u32,
    ordering: Option<String>,
}

impl TableQuery {
    pub(crate) fn new(target: TableTarget) -> Self {
        Self {
            target,
            limit: 500,
            offset: 0,
            ordering: None,
        }
    }

    pub(crate) fn set_limit(&mut self, limit: u32) -> &mut Self {
        self.limit = limit;
        self
    }

    pub(crate) fn set_offset(&mut self, offset: u32) -> &mut Self {
        self.offset = offset;
        self
    }

    pub(crate) fn offset(&self) -> u32 {
        self.offset
    }

    pub(crate) fn limit(&self) -> u32 {
        self.limit
    }

    fn sql(&self) -> String {
        let table_name = self.target.table_name.clone();
        let schema_name = self.target.schema_name.clone();
        let from_clause = format!("\"{schema_name}\".\"{table_name}\"",);
        let limit = self.limit + 1;
        let offset = self.offset;
        format!("SELECT * from {from_clause} LIMIT {limit} OFFSET {offset}")
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
    pub(crate) error_string: Option<String>,
    pub(crate) query_task: Option<Task<()>>,
    pub(crate) status: QueryStatus,
}

impl QueryState {
    pub(crate) fn new(query: QuerySource) -> Self {
        Self {
            query,
            status: QueryStatus::Idle,
            results: None,
            has_next: None,
            error_string: None,
            query_task: None,
        }
    }

    pub(crate) fn from_table_target(target: TableTarget) -> Self {
        let query = QuerySource::Table(TableQuery::new(target));
        Self {
            query,
            status: QueryStatus::Idle,
            has_next: None,
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

    pub(crate) fn page_size(&self) -> usize {
        match &self.query {
            QuerySource::Table(query) => query.limit() as usize,
            QuerySource::Sql(_) => self
                .results
                .as_ref()
                .map_or(0, |results| results.rows.len()),
        }
    }

    pub(crate) fn offset(&self) -> u32 {
        match &self.query {
            QuerySource::Table(query) => query.offset(),
            QuerySource::Sql(_) => 0,
        }
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
        let Some(next_offset) = query.offset().checked_add(query.limit()) else {
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
        let next_offset = query.offset().saturating_sub(query.limit());
        query.set_offset(next_offset);
        self.execute(cx, session);
    }

    pub(crate) fn is_loading(&self) -> bool {
        matches!(self.status, QueryStatus::Idle | QueryStatus::Running)
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
                                    QuerySource::Table(table_query) => {
                                        let limit = table_query.limit as usize;
                                        let has_next = output.rows.len() > limit;
                                        output.rows.truncate(limit);
                                        Some(has_next)
                                    }
                                    QuerySource::Sql(_) => Some(false),
                                };
                                state.results = Some(output);
                                state.status = QueryStatus::Succeeded;
                            }
                            Err(error) => {
                                state.error_string = Some(error.clone());
                                state.status = QueryStatus::Failed;
                                eprintln!("failed to execute query. Error: {error} ")
                            }
                        }
                        cx.notify();
                    }) {
                        eprint!("failed to update query state: {error}")
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
