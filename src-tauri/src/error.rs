use thiserror::Error;

#[derive(Debug, Error)]
pub enum CliplyError {
    #[error("storage is not available yet: {0}")]
    StorageUnavailable(String),
    #[error("platform API is not available yet: {0}")]
    PlatformUnavailable(String),
}

impl From<CliplyError> for String {
    fn from(error: CliplyError) -> Self {
        error.to_string()
    }
}
