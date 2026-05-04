use crate::error::CliplyError;

pub fn unavailable() -> Result<(), CliplyError> {
    Err(CliplyError::PlatformUnavailable(
        "macOS adapter is reserved for a later phase".to_string(),
    ))
}
