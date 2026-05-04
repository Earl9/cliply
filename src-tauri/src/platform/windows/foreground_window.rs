use crate::error::CliplyError;
use crate::platform::ForegroundAppInfo;

pub fn current_foreground_app() -> Result<Option<ForegroundAppInfo>, CliplyError> {
    Ok(None)
}
