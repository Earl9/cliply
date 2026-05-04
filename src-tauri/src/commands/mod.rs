use crate::models::clipboard_item::{ClipboardItemDetailDto, ClipboardItemDto};
use crate::services::{clipboard_service, database_service, paste_service};
use tauri::{AppHandle, Emitter};

#[tauri::command]
pub async fn initialize_storage(app: AppHandle) -> Result<(), String> {
    database_service::initialize(&app).map_err(Into::into)
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
