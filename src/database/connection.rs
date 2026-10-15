use crate::database::result::QueryOutput;

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

    #[error("failed to execute query: {0}")]
    ExecuteQuery(#[source] sqlx::Error),
}

#[derive(Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub(crate) struct ConnectionParameters {
    pub(crate) host: String,
    pub(crate) port: String,
    pub(crate) database: String,
    pub(crate) username: String,
    pub(crate) password: String,
    pub(crate) ssl_mode: String,
}

#[derive(Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub(crate) enum ConnectionDetails {
    Parameters(ConnectionParameters),
    Uri(String),
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

    pub async fn execute_query(&self, query: String) -> Result<QueryOutput, DatabaseError> {
        match self {
            DatabaseConnection::PostgreSql(database) => database
                .execute_query(query)
                .await
                .map_err(DatabaseError::ExecuteQuery),
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
            DatabaseBackend::PostgreSql => {
                PostgresDriver::new(ConnectionDetails::Parameters(profile.fields))
                    .await
                    .map(DatabaseConnection::PostgreSql)
                    .map_err(DatabaseError::Connect)
            }
        }
    }

    pub async fn test_connection(
        connection_details: ConnectionDetails,
        backend: DatabaseBackend,
    ) -> Result<String, DatabaseError> {
        match backend {
            DatabaseBackend::PostgreSql => PostgresDriver::test_connection(connection_details)
                .await
                .map_err(DatabaseError::Connect),
        }
    }
}
