use tauri::{AppHandle, Manager};

pub fn initialize_mock_storage(app: &AppHandle) -> tauri::Result<()> {
    let _ = app.path().app_data_dir();
    Ok(())
}
