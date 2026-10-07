use gpui::{Context, Entity, Task};

use crate::{
    database::{result::QueryOutput, session::DatabaseSession},
    workspace::TableTarget,
};

struct TableQuery {
    target: TableTarget,
    limit: u32,
    offset: u32,
    ordering: String,
}

impl TableQuery {
    fn sql(&self) -> String {
        let table_name = self.target.table_name.clone();
        let schema_name = self.target.schema_name.clone();
        let from_clause = format!("\"{schema_name}.\".\"{table_name}\"",);
        let limit = self.limit;
        format!("SELECT * from {from_clause} LIMIT {limit}")
    }
}

enum QuerySource {
    Sql { sql: String },
    // Other things like filters, pagination, etc
    Table(TableQuery),
}

impl QuerySource {
    fn sql(&self) -> String {
        match self {
            Self::Sql { sql } => sql.clone(),
            Self::Table(query) => query.sql(),
        }
    }
}

enum QueryStatus {
    Idle,
    Running,
    Succeeded,
    Failed,
}

pub(crate) struct QueryState {
    pub(crate) query: QuerySource,
    pub(crate) results: Option<QueryOutput>,
    pub(crate) error_string: Option<String>,
    pub(crate) query_task: Option<Task<()>>, // reults, state, running task, exeecution time, etc
    pub(crate) status: QueryStatus,
}

impl QueryState {
    pub(crate) fn new(query: QuerySource) -> Self {
        Self {
            query,
            status: QueryStatus::Idle,
            results: None,
            error_string: None,
            query_task: None,
        }
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
                            Ok(output) => {
                                state.results = Some(output);
                                state.status = QueryStatus::Succeeded;
                            }
                            Err(error) => {
                                state.error_string = Some(error);
                                state.status = QueryStatus::Failed;
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
    }
}
