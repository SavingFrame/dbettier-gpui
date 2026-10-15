use crate::database::{connection::ConnectionParameters, postgres::PostgresDriver};

#[derive(Debug, serde::Serialize, serde::Deserialize, Clone, Copy)]
pub enum DatabaseBackend {
    PostgreSql,
}

#[derive(serde::Serialize, serde::Deserialize, Clone)]
pub struct DatabaseConnectionProfile {
    pub uuid: String,
    pub name: String,
    pub backend: DatabaseBackend,
    #[serde(flatten)]
    pub fields: ConnectionParameters,
}

impl DatabaseConnectionProfile {
    pub fn generate_uri(&self) -> String {
        match self.backend {
            DatabaseBackend::PostgreSql => PostgresDriver::details_to_uri(
                super::connection::ConnectionDetails::Parameters(self.fields.clone()),
            ),
        }
    }
}
