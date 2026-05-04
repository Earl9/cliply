use crate::error::CliplyError;
use crate::platform::{self, ClipboardWritePayload};
use crate::services::database_service;
use rusqlite::params;
use std::thread;
use std::time::Duration;
use tauri::{AppHandle, Manager};

pub fn copy_clipboard_item(app: &AppHandle, id: String) -> Result<(), CliplyError> {
    write_item_to_clipboard(app, &id)
}

pub fn paste_clipboard_item(app: &AppHandle, id: String) -> Result<(), CliplyError> {
    write_item_to_clipboard(app, &id)?;
    increment_used_count(app, &id)?;
    hide_main_window(app);
    thread::sleep(Duration::from_millis(120));
    platform::paste_to_foreground()
}

pub fn paste_plain_text(app: &AppHandle, id: String) -> Result<(), CliplyError> {
    paste_clipboard_item(app, id)
}

fn write_item_to_clipboard(app: &AppHandle, id: &str) -> Result<(), CliplyError> {
    let text = load_item_text(app, id)?;
    platform::write_clipboard_payload(
        ClipboardWritePayload {
            text: Some(text),
            html: None,
            image_path: None,
        },
        main_window_handle(app),
    )
}

fn load_item_text(app: &AppHandle, id: &str) -> Result<String, CliplyError> {
    let connection = database_service::connect(app)?;
    let result = connection.query_row(
        "SELECT COALESCE(cf.data_text, ci.normalized_text, '')
         FROM clipboard_items ci
         LEFT JOIN clipboard_formats cf ON cf.item_id = ci.id
              AND cf.data_kind IN ('text', 'html')
              AND COALESCE(cf.data_text, '') <> ''
         WHERE ci.id = ?1
           AND ci.is_deleted = 0
           AND ci.type <> 'image'
         ORDER BY cf.priority DESC, cf.created_at ASC
         LIMIT 1",
        params![id],
        |row| row.get::<_, String>(0),
    );

    match result {
        Ok(value) if !value.is_empty() => Ok(value),
        Ok(_) => Err(CliplyError::PlatformUnavailable(
            "selected clipboard item has no text fallback".into(),
        )),
        Err(rusqlite::Error::QueryReturnedNoRows) => Err(CliplyError::PlatformUnavailable(
            "selected clipboard item was not found".into(),
        )),
        Err(error) => Err(error.into()),
    }
}

fn main_window_handle(app: &AppHandle) -> Option<isize> {
    #[cfg(target_os = "windows")]
    {
        return app
            .get_webview_window("main")
            .and_then(|window| window.hwnd().ok())
            .map(|hwnd| hwnd.0 as isize);
    }

    #[cfg(not(target_os = "windows"))]
    {
        let _ = app;
        None
    }
}

fn increment_used_count(app: &AppHandle, id: &str) -> Result<(), CliplyError> {
    let connection = database_service::connect(app)?;
    connection.execute(
        "UPDATE clipboard_items
         SET used_count = COALESCE(used_count, 0) + 1,
             updated_at = datetime('now')
         WHERE id = ?1",
        params![id],
    )?;

    Ok(())
}

fn hide_main_window(app: &AppHandle) {
    if let Some(window) = app.get_webview_window("main") {
        let _ = window.hide();
    }
}
