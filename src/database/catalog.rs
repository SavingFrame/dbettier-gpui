#[derive(Clone)]
pub enum ConstraintType {
    ForeignKey,
    Unique,
    PrimaryKey,
    Check,
    Other(String),
}

impl ConstraintType {
    pub(super) fn from_str(value: &str) -> Self {
        match value {
            "FOREIGN KEY" => ConstraintType::ForeignKey,
            "UNIQUE" => ConstraintType::Unique,
            "PRIMARY KEY" => ConstraintType::PrimaryKey,
            "CHECK" => ConstraintType::Check,
            other => ConstraintType::Other(other.to_string()),
        }
    }
}

#[derive(Clone)]
pub struct TableColumn {
    pub name: String,
    pub default: Option<String>,
    pub is_nullable: bool,
    pub data_type: String,
    pub character_maximum_length: Option<u32>,
    pub is_auto_increment: bool,
}

#[derive(Clone)]
pub struct TableConstraint {
    pub name: String,
    pub constraint_type: ConstraintType,
    pub is_deferrable: bool,
    pub nulls_distinct: Option<bool>,
}

#[derive(Clone)]
pub struct DatabaseTable {
    pub name: String,
    pub columns: Vec<TableColumn>,
    pub constraints: Vec<TableConstraint>,
}

#[derive(Clone)]
pub enum LoadState<T> {
    NotLoaded,
    Loading,
    Loaded(T),
    Failed(String),
}

#[derive(Clone)]
pub struct DatabaseSchema {
    pub name: String,
    pub tables: LoadState<Vec<DatabaseTable>>,
}
