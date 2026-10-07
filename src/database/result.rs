use std::time::Duration;

use sqlx::postgres::PgRow;

#[derive(Clone)]
pub(crate) struct ResultColumn {
    pub(crate) name: String,
    pub(crate) column_type: String,
}

pub(crate) struct QueryOutput {
    pub(crate) columns: Vec<ResultColumn>,
    pub(crate) rows: Vec<PgRow>,
    pub(crate) rows_affected: u64,
    pub(crate) elapsed: Duration,
}
