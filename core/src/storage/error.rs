use thiserror::Error;

#[derive(Error, Debug)]
pub enum StorageError {
    #[error("database error: {0}")]
    Database(String),

    #[error("record not found")]
    NotFound,

    #[error("authentication failed (tampered storage)")]
    AuthFailed,

    #[error("invalid data: {0}")]
    InvalidData(String),

    #[error("not initialized: master key required")]
    MasterKeyRequired,

    #[error("already initialized")]
    AlreadyInitialized,

    #[error("I/O error: {0}")]
    Io(String),

    #[error("migration error: {0}")]
    Migration(String),
}

impl From<rusqlite::Error> for StorageError {
    fn from(e: rusqlite::Error) -> Self {
        StorageError::Database(e.to_string())
    }
}

impl From<std::str::Utf8Error> for StorageError {
    fn from(e: std::str::Utf8Error) -> Self {
        StorageError::InvalidData(e.to_string())
    }
}
