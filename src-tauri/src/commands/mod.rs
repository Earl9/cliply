use crate::models::clipboard_item::{ClipboardItemDetailDto, ClipboardItemDto};
use crate::services::{clipboard_service, database_service};

#[tauri::command]
pub async fn initialize_storage() -> Result<(), String> {
    database_service::initialize().map_err(Into::into)
}

#[tauri::command]
pub async fn list_clipboard_items(
    query: Option<String>,
    item_type: Option<String>,
    pinned_only: Option<bool>,
    limit: Option<i64>,
    offset: Option<i64>,
) -> Result<Vec<ClipboardItemDto>, String> {
    clipboard_service::list_mock_items(query, item_type, pinned_only, limit, offset)
        .map_err(Into::into)
}

#[tauri::command]
pub async fn get_clipboard_item_detail(id: String) -> Result<ClipboardItemDetailDto, String> {
    clipboard_service::get_mock_item_detail(id).map_err(Into::into)
}

#[tauri::command]
pub async fn toggle_pin_clipboard_item(id: String) -> Result<ClipboardItemDto, String> {
    clipboard_service::toggle_pin_mock_item(id).map_err(Into::into)
}
