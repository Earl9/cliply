use thiserror::Error;

#[derive(Debug, Error)]
pub enum CliplyError {
    #[error("storage is not available yet: {0}")]
    StorageUnavailable(String),
    #[error("database error: {0}")]
    Database(#[from] rusqlite::Error),
    #[error("filesystem error: {0}")]
    Filesystem(#[from] std::io::Error),
    #[error("platform API is not available yet: {0}")]
    PlatformUnavailable(String),
}

impl From<CliplyError> for String {
    fn from(error: CliplyError) -> Self {
        error.to_string()
    }
}
