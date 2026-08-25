#[derive(Debug, serde::Serialize, serde::Deserialize, Clone, Copy)]
pub enum DatabaseBackend {
    PostgreSql,
}

#[derive(serde::Serialize, serde::Deserialize, Clone)]
pub struct DatabaseConnectionProfile {
    pub uuid: String,
    pub name: String,
    pub backend: DatabaseBackend,
    pub uri: String,
}
