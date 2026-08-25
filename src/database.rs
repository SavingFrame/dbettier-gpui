use crate::database::postgres::PostgresDriver;

mod postgres;
mod profile;
pub(crate) mod profile_storage;
pub(crate) mod profile_store;
pub(crate) mod session_store;

pub use profile::{DatabaseBackend, DatabaseConnectionProfile};

#[derive(Debug, thiserror::Error)]
pub enum DatabaseError {
    #[error("failed to connect to database: {0}")]
    Connect(#[source] sqlx::Error),

    #[error("failed to list database schemas: {0}")]
    ListSchemas(#[source] sqlx::Error),

    #[error("failed to list tables in schema {schema_name}: {source}")]
    ListTables {
        schema_name: String,
        #[source]
        source: sqlx::Error,
    },
}

#[derive(Clone)]
pub enum ConstraintType {
    ForeignKey,
    Unique,
    PrimaryKey,
    Check,
    Other(String),
}

impl ConstraintType {
    fn from_str(value: &str) -> Self {
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
pub enum DatabaseConnection {
    PostgreSql(PostgresDriver),
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
pub struct DatabaseSchema {
    pub name: String,
}

impl DatabaseConnection {
    pub async fn close(&self) {
        match self {
            DatabaseConnection::PostgreSql(database) => database.close().await,
        }
    }

    pub async fn list_schemas(&self) -> Result<Vec<DatabaseSchema>, DatabaseError> {
        match self {
            DatabaseConnection::PostgreSql(database) => database
                .list_schemas()
                .await
                .map_err(DatabaseError::ListSchemas),
        }
    }

    pub async fn list_tables(
        &self,
        schema_name: &str,
    ) -> Result<Vec<DatabaseTable>, DatabaseError> {
        match self {
            DatabaseConnection::PostgreSql(database) => database
                .list_tables(schema_name)
                .await
                .map_err(|source| DatabaseError::ListTables {
                    schema_name: schema_name.to_string(),
                    source,
                }),
        }
    }

    pub async fn from_connection_profile(
        profile: DatabaseConnectionProfile,
    ) -> Result<Self, DatabaseError> {
        match profile.backend {
            DatabaseBackend::PostgreSql => PostgresDriver::new(profile.uri)
                .await
                .map(DatabaseConnection::PostgreSql)
                .map_err(DatabaseError::Connect),
        }
    }
}
