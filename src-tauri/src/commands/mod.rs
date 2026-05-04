use crate::models::clipboard_item::{ClipboardItemDetailDto, ClipboardItemDto};
use crate::models::settings::CliplySettings;
use crate::services::{clipboard_service, database_service, paste_service, settings_service};
use crate::{shortcuts, tray};
use tauri::{AppHandle, Emitter};

#[tauri::command]
pub async fn initialize_storage(app: AppHandle) -> Result<(), String> {
    database_service::initialize(&app).map_err::<String, _>(Into::into)?;
    let cleanup =
        clipboard_service::enforce_history_retention(&app).map_err::<String, _>(Into::into)?;
    if cleanup.deleted_items > 0 {
        let _ = app.emit("clipboard-items-changed", ());
    }
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
        .map_err(Into::into)
}

#[tauri::command]
pub async fn get_clipboard_item_detail(
    app: AppHandle,
    id: String,
) -> Result<ClipboardItemDetailDto, String> {
    clipboard_service::get_clipboard_item_detail(&app, id).map_err(Into::into)
}

#[tauri::command]
pub async fn toggle_pin_clipboard_item(
    app: AppHandle,
    id: String,
) -> Result<ClipboardItemDto, String> {
    clipboard_service::toggle_pin_clipboard_item(&app, id).map_err(Into::into)
}

#[tauri::command]
pub async fn delete_clipboard_item(app: AppHandle, id: String) -> Result<(), String> {
    clipboard_service::delete_clipboard_item(&app, id).map_err::<String, _>(Into::into)?;
    let _ = app.emit("clipboard-items-changed", ());
    Ok(())
}

#[tauri::command]
pub async fn clear_clipboard_history(app: AppHandle, include_pinned: bool) -> Result<(), String> {
    clipboard_service::clear_clipboard_history(&app, include_pinned)
        .map_err::<String, _>(Into::into)?;
    let _ = app.emit("clipboard-items-changed", ());
    Ok(())
}

#[tauri::command]
pub async fn copy_clipboard_item(app: AppHandle, id: String) -> Result<(), String> {
    paste_service::copy_clipboard_item(&app, id).map_err(Into::into)
}

#[tauri::command]
pub async fn paste_clipboard_item(app: AppHandle, id: String) -> Result<(), String> {
    paste_service::paste_clipboard_item(&app, id).map_err(Into::into)
}

#[tauri::command]
pub async fn paste_plain_text(app: AppHandle, id: String) -> Result<(), String> {
    paste_service::paste_plain_text(&app, id).map_err(Into::into)
}

#[tauri::command]
pub async fn get_cliply_settings(app: AppHandle) -> Result<CliplySettings, String> {
    settings_service::get_settings(&app).map_err(Into::into)
}

#[tauri::command]
pub async fn update_cliply_settings(
    app: AppHandle,
    settings: CliplySettings,
) -> Result<CliplySettings, String> {
    shortcuts::validate_shortcut(&settings.global_shortcut).map_err(|error| error.to_string())?;
    let previous_settings =
        settings_service::get_settings(&app).map_err::<String, _>(Into::into)?;
    let updated_settings =
        settings_service::update_settings(&app, settings).map_err::<String, _>(Into::into)?;

    if previous_settings.global_shortcut != updated_settings.global_shortcut {
        shortcuts::register_default_shortcuts(&app).map_err(|error| error.to_string())?;
    }

    let cleanup =
        clipboard_service::enforce_history_retention_with_settings(&app, &updated_settings)
            .map_err::<String, _>(Into::into)?;
    if cleanup.deleted_items > 0 {
        let _ = app.emit("clipboard-items-changed", ());
    }

    tray::refresh_tray(&app).map_err(|error| error.to_string())?;
    Ok(updated_settings)
}

#[tauri::command]
pub async fn set_monitoring_paused(app: AppHandle, paused: bool) -> Result<CliplySettings, String> {
    let settings =
        settings_service::set_monitoring_paused(&app, paused).map_err::<String, _>(Into::into)?;
    tray::refresh_tray(&app).map_err(|error| error.to_string())?;
    Ok(settings)
}

#[tauri::command]
pub async fn show_main_window(app: AppHandle) -> Result<(), String> {
    crate::show_main_window(&app).map_err(|error| error.to_string())
}

#[tauri::command]
pub async fn hide_main_window(app: AppHandle) -> Result<(), String> {
    crate::hide_main_window(&app).map_err(|error| error.to_string())
}

#[tauri::command]
pub async fn toggle_main_window_pin(app: AppHandle, pinned: bool) -> Result<(), String> {
    crate::toggle_main_window_pin(&app, pinned).map_err(|error| error.to_string())
}
