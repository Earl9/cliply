use crate::error::CliplyError;

pub fn unavailable() -> Result<(), CliplyError> {
    Err(CliplyError::PlatformUnavailable(
        "Linux adapter is reserved for a later phase".to_string(),
    ))
}
