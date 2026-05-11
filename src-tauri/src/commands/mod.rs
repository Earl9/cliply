use crate::logger;
use crate::models::clipboard_item::{ClipboardItemDetailDto, ClipboardItemDto};
use crate::models::settings::CliplySettings;
use crate::services::sync_storage_provider::SyncProviderConfig;
use crate::services::{
    clipboard_service, database_service, paste_service, remote_sync_service, settings_service,
    sync_package_service,
};
use crate::{platform, shortcuts, tray};
use rusqlite::{params, OptionalExtension};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::fs::{self, File};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::Duration;
use tauri::{AppHandle, Emitter};

const CLIPLY_RELEASE_PAGE_URL: &str = "https://github.com/Earl9/cliply/releases/latest";
const CLIPLY_UPDATE_MANIFEST_URL: &str =
    "https://github.com/Earl9/cliply/releases/latest/download/latest.json";
const MODERN_INSTALLER_FILE_NAME: &str = "cliply-modern-installer.exe";

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
pub async fn get_current_install_dir(app: AppHandle) -> Result<String, String> {
    if let Some(install_dir) = platform::read_install_dir_from_registry() {
        logger::info(&app, "command.get_current_install_dir", "source=registry");
        return Ok(install_dir);
    }

    let current_exe = std::env::current_exe()
        .map_err(|error| command_error(&app, "get_current_install_dir.current_exe", error))?;
    let install_dir = current_exe
        .parent()
        .map(Path::to_path_buf)
        .ok_or_else(|| command_error(&app, "get_current_install_dir.parent", "安装目录不可用"))?;
    logger::info(
        &app,
        "command.get_current_install_dir",
        "source=current_exe",
    );
    Ok(install_dir.to_string_lossy().to_string())
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ModernUpdateDownloadRequest {
    pub url: String,
    pub sha256: String,
    pub file_name: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ModernUpdateDownloadResult {
    pub path: String,
    pub sha256: String,
    pub size_bytes: u64,
}

#[derive(Debug, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ModernUpdateDownloadProgress {
    pub downloaded_bytes: u64,
    pub total_bytes: Option<u64>,
}

#[tauri::command]
pub async fn fetch_cliply_update_manifest(app: AppHandle) -> Result<serde_json::Value, String> {
    logger::info(
        &app,
        "update_check_started",
        "source=github_latest_manifest",
    );

    let response = match ureq::get(CLIPLY_UPDATE_MANIFEST_URL)
        .set("Accept", "application/json")
        .set("Cache-Control", "no-cache")
        .set("Pragma", "no-cache")
        .timeout(Duration::from_secs(30))
        .call()
    {
        Ok(response) => response,
        Err(error) => {
            logger::error(
                &app,
                "update_check_failed",
                format!("kind=network error={}", sanitize_log_value(&error.to_string())),
            );
            return Err("检查更新失败，请检查网络后重试".to_string());
        }
    };

    let mut body = String::new();
    if let Err(error) = response.into_reader().read_to_string(&mut body) {
        logger::error(
            &app,
            "update_check_failed",
            format!("kind=read error={}", sanitize_log_value(&error.to_string())),
        );
        return Err("无法读取更新清单，请稍后重试".to_string());
    }

    let manifest = match serde_json::from_str::<serde_json::Value>(&body) {
        Ok(manifest) => manifest,
        Err(error) => {
            logger::error(
                &app,
                "update_check_failed",
                format!("kind=parse error={}", sanitize_log_value(&error.to_string())),
            );
            return Err("更新清单格式不正确，请稍后重试".to_string());
        }
    };

    logger::info(
        &app,
        "update_check_success",
        "source=github_latest_manifest",
    );
    Ok(manifest)
}

#[tauri::command]
pub async fn download_modern_update_installer(
    app: AppHandle,
    request: ModernUpdateDownloadRequest,
) -> Result<ModernUpdateDownloadResult, String> {
    logger::info(
        &app,
        "update_modern_installer_download_started",
        "asset=modern-installer",
    );
    let result = download_modern_update_installer_inner(&app, request)
        .map_err(|error| command_error(&app, "download_modern_update_installer", error))?;
    logger::info(
        &app,
        "update_modern_installer_download_success",
        format!(
            "size_bytes={} sha256={}",
            result.size_bytes,
            sanitize_log_value(&result.sha256)
        ),
    );
    Ok(result)
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LaunchModernUpdateInstallerRequest {
    pub installer_path: String,
    pub install_dir: String,
    pub source_version: String,
    pub target_version: String,
}

#[tauri::command]
pub async fn launch_modern_update_installer(
    app: AppHandle,
    request: LaunchModernUpdateInstallerRequest,
) -> Result<(), String> {
    let installer_path = PathBuf::from(&request.installer_path);
    if !installer_path.is_file() {
        return Err(command_error(
            &app,
            "launch_modern_update_installer.installer_path",
            "更新安装器不存在",
        ));
    }
    if !is_downloaded_modern_installer_path(&installer_path) {
        return Err(command_error(
            &app,
            "launch_modern_update_installer.installer_path",
            "更新安装器路径不合法",
        ));
    }

    let current_pid = std::process::id();
    let args = vec![
        "--mode".to_string(),
        "update".to_string(),
        "--install-dir".to_string(),
        request.install_dir.clone(),
        "--source-version".to_string(),
        request.source_version.clone(),
        "--target-version".to_string(),
        request.target_version.clone(),
        "--preserve-user-data".to_string(),
        "--launch-after-install".to_string(),
        "--parent-pid".to_string(),
        current_pid.to_string(),
    ];
    let safe_source_version = sanitize_log_value(&request.source_version);
    let safe_target_version = sanitize_log_value(&request.target_version);
    spawn_modern_installer(&installer_path, &args)
        .map_err(|error| command_error(&app, "launch_modern_update_installer.spawn", error))?;

    logger::info(
        &app,
        "update_modern_installer_launched",
        format!(
            "source_version={} target_version={}",
            safe_source_version, safe_target_version
        ),
    );

    app.exit(0);
    Ok(())
}

#[tauri::command]
pub async fn open_cliply_release_page(app: AppHandle) -> Result<(), String> {
    open_url(CLIPLY_RELEASE_PAGE_URL)
        .map_err(|error| command_error(&app, "open_cliply_release_page.open", error))?;
    logger::info(&app, "command.open_cliply_release_page", "ok");
    Ok(())
}

#[tauri::command]
pub async fn log_update_install_failed(app: AppHandle, version: String) -> Result<(), String> {
    logger::error(
        &app,
        "update_install_failed",
        format!("version={}", sanitize_log_value(&version)),
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

fn open_url(url: &str) -> std::io::Result<()> {
    #[cfg(target_os = "windows")]
    {
        platform::open_url(url).map_err(|error| std::io::Error::other(error.to_string()))
    }

    #[cfg(target_os = "macos")]
    {
        Command::new("open").arg(url).spawn().map(|_| ())
    }

    #[cfg(all(unix, not(target_os = "macos")))]
    {
        Command::new("xdg-open").arg(url).spawn().map(|_| ())
    }
}

fn download_modern_update_installer_inner(
    app: &AppHandle,
    request: ModernUpdateDownloadRequest,
) -> Result<ModernUpdateDownloadResult, String> {
    let expected_sha256 = normalize_sha256(&request.sha256)?;
    let file_name = sanitize_installer_file_name(&request.file_name)?;
    let update_dir = std::env::temp_dir().join("Cliply").join("updates");
    fs::create_dir_all(&update_dir).map_err(|error| error.to_string())?;
    let output_path = update_dir.join(file_name);
    if output_path.exists() {
        fs::remove_file(&output_path).map_err(|error| error.to_string())?;
    }

    let response = ureq::get(&request.url)
        .timeout(Duration::from_secs(120))
        .call()
        .map_err(|_| "更新安装器下载失败，请检查网络后重试".to_string())?;
    let total_bytes = response
        .header("Content-Length")
        .and_then(|value| value.parse::<u64>().ok());
    let mut reader = response.into_reader();
    let mut file = File::create(&output_path).map_err(|error| error.to_string())?;
    let mut hasher = Sha256::new();
    let mut buffer = [0u8; 64 * 1024];
    let mut downloaded_bytes = 0u64;

    loop {
        let bytes_read = reader
            .read(&mut buffer)
            .map_err(|error| error.to_string())?;
        if bytes_read == 0 {
            break;
        }
        file.write_all(&buffer[..bytes_read])
            .map_err(|error| error.to_string())?;
        hasher.update(&buffer[..bytes_read]);
        downloaded_bytes += bytes_read as u64;
        let _ = app.emit(
            "modern-update-download-progress",
            ModernUpdateDownloadProgress {
                downloaded_bytes,
                total_bytes,
            },
        );
    }
    file.flush().map_err(|error| error.to_string())?;

    let actual_sha256 = format!("{:x}", hasher.finalize());
    if actual_sha256 != expected_sha256 {
        let _ = fs::remove_file(&output_path);
        logger::error(
            app,
            "update_modern_installer_checksum_failed",
            format!(
                "expected_sha256={} actual_sha256={}",
                sanitize_log_value(&expected_sha256),
                sanitize_log_value(&actual_sha256)
            ),
        );
        return Err("更新包校验失败".to_string());
    }

    Ok(ModernUpdateDownloadResult {
        path: output_path.to_string_lossy().to_string(),
        sha256: actual_sha256,
        size_bytes: downloaded_bytes,
    })
}

fn is_downloaded_modern_installer_path(path: &Path) -> bool {
    let update_dir = std::env::temp_dir().join("Cliply").join("updates");
    let Ok(canonical_path) = path.canonicalize() else {
        return false;
    };
    let Ok(canonical_update_dir) = update_dir.canonicalize() else {
        return false;
    };
    canonical_path.starts_with(canonical_update_dir)
        && canonical_path
            .file_name()
            .and_then(|name| name.to_str())
            .map(|name| name.to_ascii_lowercase().ends_with("-modern-installer.exe"))
            .unwrap_or(false)
}

fn normalize_sha256(value: &str) -> Result<String, String> {
    let normalized = value.trim().to_ascii_lowercase();
    if normalized.len() == 64
        && normalized
            .chars()
            .all(|character| character.is_ascii_hexdigit())
    {
        Ok(normalized)
    } else {
        Err("更新清单中的 SHA256 不合法".to_string())
    }
}

fn sanitize_installer_file_name(value: &str) -> Result<String, String> {
    let file_name = Path::new(value)
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or(MODERN_INSTALLER_FILE_NAME);
    if !file_name.to_ascii_lowercase().ends_with(".exe") {
        return Err("更新安装器文件名不合法".to_string());
    }
    if !file_name
        .to_ascii_lowercase()
        .ends_with("-modern-installer.exe")
    {
        return Err("更新清单中的安装器不是 Modern Installer".to_string());
    }
    let sanitized: String = file_name
        .chars()
        .filter(|character| {
            character.is_ascii_alphanumeric() || matches!(character, '.' | '-' | '_')
        })
        .take(160)
        .collect();
    if sanitized.is_empty() {
        Ok(MODERN_INSTALLER_FILE_NAME.to_string())
    } else {
        Ok(sanitized)
    }
}

fn sanitize_log_value(value: &str) -> String {
    value
        .chars()
        .filter(|character| {
            character.is_ascii_alphanumeric() || matches!(character, '.' | '-' | '_')
        })
        .take(64)
        .collect()
}

#[cfg(target_os = "windows")]
fn spawn_modern_installer(path: &Path, args: &[String]) -> std::io::Result<()> {
    use std::ffi::OsString;
    use windows_sys::Win32::Foundation::HWND;
    use windows_sys::Win32::UI::Shell::ShellExecuteW;
    use windows_sys::Win32::UI::WindowsAndMessaging::SW_SHOWNORMAL;

    let operation = wide_null("runas");
    let file = wide_null_os(path.as_os_str());
    let parameters = wide_null_os(&OsString::from(
        args.iter()
            .map(|arg| quote_windows_arg(arg))
            .collect::<Vec<_>>()
            .join(" "),
    ));
    let result = unsafe {
        ShellExecuteW(
            std::ptr::null_mut() as HWND,
            operation.as_ptr(),
            file.as_ptr(),
            parameters.as_ptr(),
            std::ptr::null(),
            SW_SHOWNORMAL,
        )
    } as isize;

    if result > 32 {
        Ok(())
    } else {
        Err(std::io::Error::other(format!(
            "ShellExecuteW failed: {result}"
        )))
    }
}

#[cfg(not(target_os = "windows"))]
fn spawn_modern_installer(path: &Path, args: &[String]) -> std::io::Result<()> {
    Command::new(path).args(args).spawn().map(|_| ())
}

#[cfg(target_os = "windows")]
fn quote_windows_arg(value: &str) -> String {
    if value.is_empty() {
        return "\"\"".to_string();
    }
    if !value
        .chars()
        .any(|character| character.is_whitespace() || character == '"')
    {
        return value.to_string();
    }

    let mut quoted = String::from("\"");
    let mut backslashes = 0usize;
    for character in value.chars() {
        if character == '\\' {
            backslashes += 1;
            continue;
        }
        if character == '"' {
            quoted.push_str(&"\\".repeat(backslashes * 2 + 1));
            quoted.push('"');
            backslashes = 0;
            continue;
        }
        quoted.push_str(&"\\".repeat(backslashes));
        backslashes = 0;
        quoted.push(character);
    }
    quoted.push_str(&"\\".repeat(backslashes * 2));
    quoted.push('"');
    quoted
}

#[cfg(target_os = "windows")]
fn wide_null(value: &str) -> Vec<u16> {
    use std::os::windows::ffi::OsStrExt;
    std::ffi::OsStr::new(value)
        .encode_wide()
        .chain(Some(0))
        .collect()
}

#[cfg(target_os = "windows")]
fn wide_null_os(value: &std::ffi::OsStr) -> Vec<u16> {
    use std::os::windows::ffi::OsStrExt;
    value.encode_wide().chain(Some(0)).collect()
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
