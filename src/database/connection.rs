use super::{
    catalog::{DatabaseSchema, DatabaseTable},
    postgres::PostgresDriver,
    profile::{DatabaseBackend, DatabaseConnectionProfile},
};

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
pub enum DatabaseConnection {
    PostgreSql(PostgresDriver),
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
