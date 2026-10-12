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
    pub(crate) fn profile_id(&self) -> &str {
        match self {
            Target::Table(table_target) => &table_target.profile_id,
            Target::Sql(sql_target) => &sql_target.profile_id,
        }
    }
}
