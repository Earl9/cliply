use crate::logger;
use crate::models::clipboard_item::{ClipboardItemDetailDto, ClipboardItemDto};
use crate::models::settings::CliplySettings;
use crate::services::sync_storage_provider::SyncProviderConfig;
use crate::services::{
    clipboard_service, database_service, paste_service, remote_sync_service, settings_service,
    sync_package_service,
};
use crate::{shortcuts, tray};
use tauri::{AppHandle, Emitter};

#[tauri::command]
pub async fn initialize_storage(app: AppHandle) -> Result<(), String> {
    database_service::initialize(&app)
        .map_err(|error| command_error(&app, "initialize_storage", error))?;
    let cleanup = clipboard_service::enforce_history_retention(&app)
        .map_err(|error| command_error(&app, "initialize_storage.retention", error))?;
    if cleanup.deleted_items > 0 {
        let _ = app.emit("clipboard-items-changed", ());
    }
    logger::info(&app, "command.initialize_storage", "ok");
    Ok(())
}

#[tauri::command]
pub async fn list_clipboard_items(
    app: AppHandle,
    query: Option<String>,
    item_type: Option<String>,
    pinned_only: Option<bool>,
    limit: Option<i64>,
    offset: Option<i64>,
) -> Result<Vec<ClipboardItemDto>, String> {
    clipboard_service::list_clipboard_items(&app, query, item_type, pinned_only, limit, offset)
        .map_err(|error| command_error(&app, "list_clipboard_items", error))
}

#[tauri::command]
pub async fn get_clipboard_item_detail(
    app: AppHandle,
    id: String,
) -> Result<ClipboardItemDetailDto, String> {
    clipboard_service::get_clipboard_item_detail(&app, id)
        .map_err(|error| command_error(&app, "get_clipboard_item_detail", error))
}

#[tauri::command]
pub async fn toggle_pin_clipboard_item(
    app: AppHandle,
    id: String,
) -> Result<ClipboardItemDto, String> {
    clipboard_service::toggle_pin_clipboard_item(&app, id)
        .map_err(|error| command_error(&app, "toggle_pin_clipboard_item", error))
}

#[tauri::command]
pub async fn delete_clipboard_item(app: AppHandle, id: String) -> Result<(), String> {
    clipboard_service::delete_clipboard_item(&app, id)
        .map_err(|error| command_error(&app, "delete_clipboard_item", error))?;
    let _ = app.emit("clipboard-items-changed", ());
    Ok(())
}

#[tauri::command]
pub async fn clear_clipboard_history(app: AppHandle, include_pinned: bool) -> Result<(), String> {
    clipboard_service::clear_clipboard_history(&app, include_pinned)
        .map_err(|error| command_error(&app, "clear_clipboard_history", error))?;
    let _ = app.emit("clipboard-items-changed", ());
    Ok(())
}

#[tauri::command]
pub async fn export_sync_package(
    app: AppHandle,
    path: String,
    password: String,
) -> Result<(), String> {
    sync_package_service::export_sync_package(&app, path, password)
        .map_err(|error| command_error(&app, "export_sync_package", error))?;
    logger::info(&app, "command.export_sync_package", "ok");
    Ok(())
}

#[tauri::command]
pub async fn import_sync_package(
    app: AppHandle,
    path: String,
    password: String,
) -> Result<sync_package_service::SyncImportResult, String> {
    let result = sync_package_service::import_sync_package(&app, path, password)
        .map_err(|error| command_error(&app, "import_sync_package", error))?;
    let _ = app.emit("clipboard-items-changed", ());
    logger::info(
        &app,
        "command.import_sync_package",
        format!(
            "imported={} updated={} skipped={} deleted={} conflicted={}",
            result.imported_count,
            result.updated_count,
            result.skipped_count,
            result.deleted_count,
            result.conflicted_count
        ),
    );
    Ok(result)
}

#[tauri::command]
pub async fn get_sync_package_status(
    app: AppHandle,
) -> Result<sync_package_service::SyncPackageStatus, String> {
    sync_package_service::get_sync_package_status(&app)
        .map_err(|error| command_error(&app, "get_sync_package_status", error))
}

#[tauri::command]
pub async fn get_remote_sync_status(
    app: AppHandle,
) -> Result<remote_sync_service::RemoteSyncStatus, String> {
    remote_sync_service::get_remote_sync_status(&app)
        .map_err(|error| command_error(&app, "get_remote_sync_status", error))
}

#[tauri::command]
pub async fn set_remote_sync_provider(
    app: AppHandle,
    config: SyncProviderConfig,
) -> Result<remote_sync_service::RemoteSyncStatus, String> {
    remote_sync_service::set_remote_sync_provider(&app, config)
        .map_err(|error| command_error(&app, "set_remote_sync_provider", error))
}

#[tauri::command]
pub async fn update_auto_sync_config(
    app: AppHandle,
    enabled: bool,
    interval_minutes: u64,
    password: Option<String>,
) -> Result<remote_sync_service::RemoteSyncStatus, String> {
    let status =
        remote_sync_service::update_auto_sync_config(&app, enabled, interval_minutes, password)
            .map_err(|error| command_error(&app, "update_auto_sync_config", error))?;
    let _ = app.emit("remote-sync-status-changed", ());
    Ok(status)
}

#[tauri::command]
pub async fn clear_auto_sync_password(
    app: AppHandle,
) -> Result<remote_sync_service::RemoteSyncStatus, String> {
    let status = remote_sync_service::clear_auto_sync_password(&app)
        .map_err(|error| command_error(&app, "clear_auto_sync_password", error))?;
    let _ = app.emit("remote-sync-status-changed", ());
    Ok(status)
}

#[tauri::command]
pub async fn sync_with_remote_now(
    app: AppHandle,
    password: Option<String>,
) -> Result<remote_sync_service::RemoteSyncResult, String> {
    let result = remote_sync_service::sync_with_remote_now(&app, password)
        .map_err(|error| command_error(&app, "sync_with_remote_now", error))?;
    let _ = app.emit("clipboard-items-changed", ());
    let _ = app.emit("remote-sync-status-changed", ());
    logger::info(
        &app,
        "command.sync_with_remote_now",
        format!(
            "exported={} imported={} updated={} deleted={} conflicted={} snapshots={}",
            result.exported_count,
            result.imported_count,
            result.updated_count,
            result.deleted_count,
            result.conflicted_count,
            result.snapshot_count
        ),
    );
    Ok(result)
}

#[tauri::command]
pub async fn export_to_remote_sync_folder(
    app: AppHandle,
    password: String,
) -> Result<remote_sync_service::RemoteSyncResult, String> {
    let result = remote_sync_service::export_to_remote_sync_folder(&app, password)
        .map_err(|error| command_error(&app, "export_to_remote_sync_folder", error))?;
    logger::info(
        &app,
        "command.export_to_remote_sync_folder",
        format!("snapshots={}", result.snapshot_count),
    );
    Ok(result)
}

#[tauri::command]
pub async fn import_from_remote_sync_folder(
    app: AppHandle,
    password: String,
) -> Result<remote_sync_service::RemoteSyncResult, String> {
    let result = remote_sync_service::import_from_remote_sync_folder(&app, password)
        .map_err(|error| command_error(&app, "import_from_remote_sync_folder", error))?;
    let _ = app.emit("clipboard-items-changed", ());
    logger::info(
        &app,
        "command.import_from_remote_sync_folder",
        format!(
            "imported={} updated={} skipped={} deleted={} conflicted={} snapshots={}",
            result.imported_count,
            result.updated_count,
            result.skipped_count,
            result.deleted_count,
            result.conflicted_count,
            result.snapshot_count
        ),
    );
    Ok(result)
}

#[tauri::command]
pub async fn copy_clipboard_item(app: AppHandle, id: String) -> Result<(), String> {
    paste_service::copy_clipboard_item(&app, id)
        .map_err(|error| command_error(&app, "copy_clipboard_item", error))
}

#[tauri::command]
pub async fn paste_clipboard_item(app: AppHandle, id: String) -> Result<(), String> {
    paste_service::paste_clipboard_item(&app, id)
        .map_err(|error| command_error(&app, "paste_clipboard_item", error))
}

#[tauri::command]
pub async fn paste_plain_text(app: AppHandle, id: String) -> Result<(), String> {
    paste_service::paste_plain_text(&app, id)
        .map_err(|error| command_error(&app, "paste_plain_text", error))
}

#[tauri::command]
pub async fn get_cliply_settings(app: AppHandle) -> Result<CliplySettings, String> {
    settings_service::get_settings(&app)
        .map_err(|error| command_error(&app, "get_cliply_settings", error))
}

#[tauri::command]
pub async fn update_cliply_settings(
    app: AppHandle,
    settings: CliplySettings,
) -> Result<CliplySettings, String> {
    shortcuts::validate_shortcut(&settings.global_shortcut)
        .map_err(|error| command_error(&app, "update_cliply_settings.validate_shortcut", error))?;
    let updated_settings = settings_service::update_settings(&app, settings)
        .map_err(|error| command_error(&app, "update_cliply_settings.save", error))?;

    let cleanup =
        clipboard_service::enforce_history_retention_with_settings(&app, &updated_settings)
            .map_err(|error| command_error(&app, "update_cliply_settings.retention", error))?;
    if cleanup.deleted_items > 0 {
        let _ = app.emit("clipboard-items-changed", ());
    }

    tray::refresh_tray(&app)
        .map_err(|error| command_error(&app, "update_cliply_settings.tray", error))?;
    Ok(updated_settings)
}

#[tauri::command]
pub async fn check_global_shortcut(
    app: AppHandle,
    shortcut: String,
    current_shortcut: Option<String>,
) -> Result<shortcuts::ShortcutCheck, String> {
    Ok(shortcuts::check_shortcut(
        &app,
        &shortcut,
        current_shortcut.as_deref(),
    ))
}

#[tauri::command]
pub async fn set_monitoring_paused(app: AppHandle, paused: bool) -> Result<CliplySettings, String> {
    let settings = settings_service::set_monitoring_paused(&app, paused)
        .map_err(|error| command_error(&app, "set_monitoring_paused", error))?;
    tray::refresh_tray(&app)
        .map_err(|error| command_error(&app, "set_monitoring_paused.tray", error))?;
    Ok(settings)
}

#[tauri::command]
pub async fn show_main_window(app: AppHandle) -> Result<(), String> {
    crate::show_main_window(&app).map_err(|error| command_error(&app, "show_main_window", error))
}

#[tauri::command]
pub async fn hide_main_window(app: AppHandle) -> Result<(), String> {
    crate::hide_main_window(&app).map_err(|error| command_error(&app, "hide_main_window", error))
}

#[tauri::command]
pub async fn minimize_main_window(app: AppHandle) -> Result<(), String> {
    crate::minimize_main_window(&app)
        .map_err(|error| command_error(&app, "minimize_main_window", error))
}

#[tauri::command]
pub async fn toggle_main_window_pin(app: AppHandle, pinned: bool) -> Result<(), String> {
    crate::toggle_main_window_pin(&app, pinned)
        .map_err(|error| command_error(&app, "toggle_main_window_pin", error))
}

#[tauri::command]
pub async fn get_debug_info(app: AppHandle) -> Result<serde_json::Value, String> {
    let log_path = logger::log_path(&app)
        .map_err(|error| command_error(&app, "get_debug_info.log_path", error))?;
    let database_path = crate::db::database_path(&app)
        .map_err(|error| command_error(&app, "get_debug_info.database_path", error))?;
    let data_dir = log_path
        .parent()
        .map(|path| path.to_path_buf())
        .unwrap_or_else(|| {
            database_path
                .parent()
                .unwrap_or(&database_path)
                .to_path_buf()
        });
    Ok(serde_json::json!({
        "logPath": log_path.to_string_lossy(),
        "databasePath": database_path.to_string_lossy(),
        "dataDir": data_dir.to_string_lossy(),
    }))
}

fn command_error(app: &AppHandle, command: &str, error: impl std::fmt::Display) -> String {
    logger::error(app, command, &error);
    error.to_string()
}
