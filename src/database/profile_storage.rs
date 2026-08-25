use std::{io, path::PathBuf};

use super::DatabaseConnectionProfile;

#[derive(Debug, thiserror::Error)]
pub enum StorageError {
    #[error("failed to read database storage file {path}: {source}")]
    ReadFile {
        path: PathBuf,
        #[source]
        source: io::Error,
    },

    #[error("failed to write database storage file {path}: {source}")]
    WriteFile {
        path: PathBuf,
        #[source]
        source: io::Error,
    },

    #[error("JSON storage error: {0}")]
    Json(#[from] serde_json::Error),
}

pub trait DatabaseStorage {
    fn load(&self) -> Result<Vec<DatabaseConnectionProfile>, StorageError>;
    fn save(&self, profiles: &[DatabaseConnectionProfile]) -> Result<(), StorageError>;
}

pub struct FileDatabaseStorage {
    path: PathBuf,
}

impl FileDatabaseStorage {
    pub fn new(path: PathBuf) -> Self {
        Self { path }
    }
}

impl DatabaseStorage for FileDatabaseStorage {
    fn load(&self) -> Result<Vec<DatabaseConnectionProfile>, StorageError> {
        let contents = match std::fs::read_to_string(&self.path) {
            Ok(contents) => contents,
            Err(source) if source.kind() == io::ErrorKind::NotFound => return Ok(Vec::new()),
            Err(source) => {
                return Err(StorageError::ReadFile {
                    path: self.path.clone(),
                    source,
                });
            }
        };

        Ok(serde_json::from_str(&contents)?)
    }

    fn save(&self, profiles: &[DatabaseConnectionProfile]) -> Result<(), StorageError> {
        let contents = serde_json::to_string_pretty(profiles)?;

        std::fs::write(&self.path, contents).map_err(|source| StorageError::WriteFile {
            path: self.path.clone(),
            source,
        })
    }
}
