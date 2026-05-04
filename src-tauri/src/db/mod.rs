use crate::error::CliplyError;
use crate::services::database_service;
use tauri::{AppHandle, Manager};

pub fn initialize_mock_storage(app: &AppHandle) -> tauri::Result<()> {
    let _ = app.path().app_data_dir();
    Ok(())
}

pub fn database_path(app: &AppHandle) -> Result<std::path::PathBuf, CliplyError> {
    let app_data_dir = app
        .path()
        .app_data_dir()
        .map_err(|error| CliplyError::StorageUnavailable(error.to_string()))?;
    std::fs::create_dir_all(&app_data_dir)?;
    Ok(app_data_dir.join("cliply.db"))
}

pub fn initialize_storage(app: &AppHandle) -> Result<(), CliplyError> {
    database_service::initialize(app)
}
