use crate::logger;
use crate::models::clipboard_item::{ClipboardItemDetailDto, ClipboardItemDto};
use crate::models::settings::CliplySettings;
use crate::services::sync_storage_provider::SyncProviderConfig;
use crate::services::{
    clipboard_service, database_service, paste_service, remote_sync_service, settings_service,
    sync_package_service, update_check_service,
};
use crate::{platform, shortcuts, tray};
use rusqlite::{params, OptionalExtension};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
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
    logger::info(
        &app,
        "command.set_remote_sync_provider",
        format!("provider_type={}", provider_type(&config)),
    );
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
    logger::info(
        &app,
        "command.update_auto_sync_config",
        format!(
            "enabled={} interval_minutes={} password_supplied={}",
            enabled,
            interval_minutes,
            password
                .as_deref()
                .is_some_and(|value| !value.trim().is_empty())
        ),
    );
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
    logger::info(&app, "command.clear_auto_sync_password", "requested");
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
    let item_id = id.clone();
    paste_service::copy_clipboard_item(&app, id)
        .map_err(|error| command_error(&app, "copy_clipboard_item", error))?;
    logger::info(
        &app,
        "command.copy_clipboard_item",
        format!("item_id={item_id}"),
    );
    Ok(())
}

#[tauri::command]
pub async fn paste_clipboard_item(app: AppHandle, id: String) -> Result<(), String> {
    let item_id = id.clone();
    paste_service::paste_clipboard_item(&app, id)
        .map_err(|error| command_error(&app, "paste_clipboard_item", error))?;
    logger::info(
        &app,
        "command.paste_clipboard_item",
        format!("item_id={item_id}"),
    );
    Ok(())
}

#[tauri::command]
pub async fn paste_plain_text(app: AppHandle, id: String) -> Result<(), String> {
    let item_id = id.clone();
    paste_service::paste_plain_text(&app, id)
        .map_err(|error| command_error(&app, "paste_plain_text", error))?;
    logger::info(
        &app,
        "command.paste_plain_text",
        format!("item_id={item_id}"),
    );
    Ok(())
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
    logger::info(
        &app,
        "command.update_cliply_settings",
        format!(
            "max_history_items={} auto_delete_days={} launch_at_startup={} start_minimized={} pause_monitoring={}",
            updated_settings.max_history_items,
            updated_settings.auto_delete_days,
            updated_settings.launch_at_startup,
            updated_settings.start_minimized,
            updated_settings.pause_monitoring
        ),
    );
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
    let data_dir = data_dir_from_paths(&log_path, &database_path);
    let log_dir = log_path
        .parent()
        .map(Path::to_path_buf)
        .unwrap_or_else(|| data_dir.clone());
    let database_size_bytes = fs::metadata(&database_path)
        .map(|metadata| metadata.len())
        .unwrap_or(0);
    let diagnostics = read_database_diagnostics(&app).unwrap_or_default();
    let recent_error = read_recent_error(&log_path);

    Ok(serde_json::json!({
        "appVersion": env!("CARGO_PKG_VERSION"),
        "logPath": log_path.to_string_lossy(),
        "logDir": log_dir.to_string_lossy(),
        "databasePath": database_path.to_string_lossy(),
        "dataDir": data_dir.to_string_lossy(),
        "databaseSizeBytes": database_size_bytes,
        "historyCount": diagnostics.history_count,
        "lastSyncedAt": diagnostics.last_synced_at,
        "lastSyncStatus": diagnostics.last_sync_status,
        "lastSyncError": diagnostics.last_sync_error.as_deref().map(redact_diagnostic_message),
        "recentError": recent_error,
    }))
}

#[tauri::command]
pub async fn open_log_directory(app: AppHandle) -> Result<(), String> {
    let log_path = logger::log_path(&app)
        .map_err(|error| command_error(&app, "open_log_directory.log_path", error))?;
    let log_dir = log_path
        .parent()
        .map(Path::to_path_buf)
        .ok_or_else(|| command_error(&app, "open_log_directory.path", "日志目录不可用"))?;
    fs::create_dir_all(&log_dir)
        .map_err(|error| command_error(&app, "open_log_directory.create_dir", error))?;

    open_directory(&log_dir)
        .map_err(|error| command_error(&app, "open_log_directory.open", error))?;
    logger::info(
        &app,
        "command.open_log_directory",
        format!("path={}", log_dir.to_string_lossy()),
    );
    Ok(())
}

#[tauri::command]
pub async fn get_system_theme_colors(app: AppHandle) -> Result<serde_json::Value, String> {
    let colors = platform::theme_adapter::read_system_theme_colors()
        .map_err(|error| command_error(&app, "get_system_theme_colors", error))?;
    logger::info(
        &app,
        "command.get_system_theme_colors",
        format!("status={} source={}", colors.status, colors.source),
    );
    serde_json::to_value(colors)
        .map_err(|error| command_error(&app, "get_system_theme_colors.serialize", error))
}

#[tauri::command]
pub async fn check_for_updates(
    app: AppHandle,
) -> Result<update_check_service::UpdateCheckResult, String> {
    let worker_app = app.clone();
    tauri::async_runtime::spawn_blocking(move || {
        update_check_service::check_for_updates(&worker_app)
    })
    .await
    .map_err(|error| command_error(&app, "check_for_updates.join", error))?
    .map_err(|error| command_error(&app, "check_for_updates", error))
}

#[tauri::command]
pub async fn open_release_page(app: AppHandle, url: String) -> Result<(), String> {
    update_check_service::open_release_page(url)
        .map_err(|error| command_error(&app, "open_release_page", error))
}

fn command_error(app: &AppHandle, command: &str, error: impl std::fmt::Display) -> String {
    let raw = error.to_string();
    logger::error(app, command, &raw);
    user_friendly_error(command, &raw)
}

#[derive(Default)]
struct DebugDatabaseDiagnostics {
    history_count: i64,
    last_synced_at: Option<String>,
    last_sync_status: Option<String>,
    last_sync_error: Option<String>,
}

fn read_database_diagnostics(app: &AppHandle) -> Result<DebugDatabaseDiagnostics, String> {
    let connection = database_service::connect(app).map_err(|error| error.to_string())?;
    let history_count = connection
        .query_row(
            "SELECT COUNT(*)
             FROM clipboard_items
             WHERE is_deleted = 0 AND deleted_at IS NULL",
            [],
            |row| row.get(0),
        )
        .unwrap_or(0);
    Ok(DebugDatabaseDiagnostics {
        history_count,
        last_synced_at: get_sync_state_value(&connection, "remote_sync_last_synced_at")?,
        last_sync_status: get_sync_state_value(&connection, "remote_sync_last_status")?,
        last_sync_error: get_sync_state_value(&connection, "remote_sync_last_error")?
            .filter(|value| !value.trim().is_empty()),
    })
}

fn get_sync_state_value(
    connection: &rusqlite::Connection,
    key: &str,
) -> Result<Option<String>, String> {
    connection
        .query_row(
            "SELECT value FROM sync_state WHERE key = ?1",
            params![key],
            |row| row.get(0),
        )
        .optional()
        .map_err(|error| error.to_string())
}

fn read_recent_error(log_path: &Path) -> Option<String> {
    let content = fs::read_to_string(log_path).ok()?;
    content
        .lines()
        .rev()
        .find(|line| line.contains(" ERROR "))
        .map(redact_diagnostic_message)
}

fn redact_diagnostic_message(message: &str) -> String {
    logger::sanitize_diagnostic_message(message)
        .chars()
        .take(500)
        .collect()
}

fn data_dir_from_paths(log_path: &Path, database_path: &Path) -> PathBuf {
    log_path.parent().map(Path::to_path_buf).unwrap_or_else(|| {
        database_path
            .parent()
            .unwrap_or(database_path)
            .to_path_buf()
    })
}

fn open_directory(path: &Path) -> std::io::Result<()> {
    #[cfg(target_os = "windows")]
    {
        use std::os::windows::process::CommandExt;
        const CREATE_NO_WINDOW: u32 = 0x08000000;
        Command::new("explorer")
            .arg(path)
            .creation_flags(CREATE_NO_WINDOW)
            .spawn()
            .map(|_| ())
    }

    #[cfg(target_os = "macos")]
    {
        Command::new("open").arg(path).spawn().map(|_| ())
    }

    #[cfg(all(unix, not(target_os = "macos")))]
    {
        Command::new("xdg-open").arg(path).spawn().map(|_| ())
    }
}

fn provider_type(config: &SyncProviderConfig) -> &'static str {
    match config {
        SyncProviderConfig::Disabled => "disabled",
        SyncProviderConfig::LocalFolder { .. } => "local-folder",
        SyncProviderConfig::Webdav { .. } => "webdav",
        SyncProviderConfig::Ftp { .. } => "ftp",
        SyncProviderConfig::Sftp { .. } => "sftp",
        SyncProviderConfig::S3 { .. } => "s3",
    }
}

fn user_friendly_error(command: &str, raw: &str) -> String {
    let lower = raw.to_ascii_lowercase();

    if lower.contains("密码错误")
        || lower.contains("decrypt")
        || lower.contains("wrong password")
        || lower.contains("aead")
    {
        return "同步密码错误或同步包已损坏。".to_string();
    }

    if lower.contains("同步包版本不兼容")
        || lower.contains("加密方式不兼容")
        || lower.contains("version")
    {
        return "同步包版本不兼容。".to_string();
    }

    if lower.contains("文件格式不正确")
        || lower.contains("file format")
        || lower.contains("manifest 格式")
        || lower.contains("expected value")
    {
        return "同步包格式不正确或文件已损坏。".to_string();
    }

    if lower.contains("rolled back") || lower.contains("已回滚") {
        return "导入失败，数据库已回滚，未污染本地历史。".to_string();
    }

    if lower.contains("webdav")
        || lower.contains("ftp")
        || lower.contains("ftps")
        || lower.contains("远程")
        || lower.contains("目录")
        || lower.contains("connect")
        || lower.contains("login")
    {
        return "远程目录不可访问，请检查地址、账号权限和网络连接。".to_string();
    }

    if command.contains("paste") || lower.contains("paste") || lower.contains("foreground") {
        return "目标窗口无法自动粘贴，内容已尽量复制到剪贴板。".to_string();
    }

    if lower.contains("not found") || lower.contains("no rows") {
        return "记录不存在或已被删除，请刷新列表后重试。".to_string();
    }

    if command.contains("copy") || lower.contains("clipboard") {
        return "剪贴板写入失败，请稍后重试。".to_string();
    }

    if lower.contains("image") || lower.contains("图片") || lower.contains("bmp") {
        return "图片读取失败，原图可能已被移动或删除。".to_string();
    }

    if lower.contains("database")
        || lower.contains("sqlite")
        || lower.contains("readonly")
        || lower.contains("locked")
        || lower.contains("constraint")
    {
        return "数据库写入失败，请检查磁盘空间、文件权限或稍后重试。".to_string();
    }

    raw.to_string()
}
