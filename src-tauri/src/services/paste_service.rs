use crate::error::CliplyError;
use crate::logger;
use crate::platform::{self, ClipboardWritePayload};
use crate::services::{database_service, settings_service};
use rusqlite::params;
use std::thread;
use std::time::Duration;
use tauri::{AppHandle, Manager};

pub fn copy_clipboard_item(app: &AppHandle, id: String) -> Result<(), CliplyError> {
    write_item_to_clipboard(app, &id)?;
    logger::info(app, "clipboard_copy", format!("item_id={id}"));
    Ok(())
}

pub fn paste_clipboard_item(app: &AppHandle, id: String) -> Result<(), CliplyError> {
    write_item_to_clipboard(app, &id)?;
    increment_used_count(app, &id)?;
    prepare_paste_target(app);
    platform::paste_to_foreground()?;
    logger::info(app, "clipboard_paste", format!("item_id={id} mode=rich"));
    Ok(())
}

pub fn paste_plain_text(app: &AppHandle, id: String) -> Result<(), CliplyError> {
    let text = load_item_text(app, &id)?;
    platform::write_clipboard_payload(
        ClipboardWritePayload {
            text: Some(text),
            html: None,
            image_path: None,
        },
        main_window_handle(app),
    )?;
    increment_used_count(app, &id)?;
    prepare_paste_target(app);
    platform::paste_to_foreground()?;
    logger::info(app, "clipboard_paste", format!("item_id={id} mode=plain"));
    Ok(())
}

fn prepare_paste_target(app: &AppHandle) {
    let close_after_paste = settings_service::get_settings(app)
        .map(|settings| settings.close_after_paste)
        .unwrap_or(true);

    if close_after_paste {
        hide_main_window(app);
    }

    thread::sleep(Duration::from_millis(80));

    let restored = platform::restore_paste_target();
    logger::info(
        app,
        "paste_target_restore",
        format!("restored={restored} close_after_paste={close_after_paste}"),
    );
    if restored {
        thread::sleep(Duration::from_millis(80));
    } else if close_after_paste {
        thread::sleep(Duration::from_millis(80));
    }
}

fn write_item_to_clipboard(app: &AppHandle, id: &str) -> Result<(), CliplyError> {
    let payload = load_item_payload(app, id)?;
    platform::write_clipboard_payload(payload, main_window_handle(app))
}

fn load_item_payload(app: &AppHandle, id: &str) -> Result<ClipboardWritePayload, CliplyError> {
    let connection = database_service::connect(app)?;
    let item_type = connection.query_row(
        "SELECT type
         FROM clipboard_items
         WHERE id = ?1
           AND is_deleted = 0
           AND deleted_at IS NULL",
        params![id],
        |row| row.get::<_, String>(0),
    );

    match item_type {
        Ok(item_type) if item_type == "image" => Ok(ClipboardWritePayload {
            text: None,
            html: None,
            image_path: Some(load_item_image_path(&connection, id)?),
        }),
        Ok(_) => Ok(ClipboardWritePayload {
            text: Some(load_item_text_with_connection(&connection, id)?),
            html: None,
            image_path: None,
        }),
        Err(rusqlite::Error::QueryReturnedNoRows) => Err(CliplyError::PlatformUnavailable(
            "selected clipboard item was not found".into(),
        )),
        Err(error) => Err(error.into()),
    }
}

fn load_item_text(app: &AppHandle, id: &str) -> Result<String, CliplyError> {
    let connection = database_service::connect(app)?;
    load_item_text_with_connection(&connection, id)
}

fn load_item_text_with_connection(
    connection: &rusqlite::Connection,
    id: &str,
) -> Result<String, CliplyError> {
    let result = connection.query_row(
        "SELECT COALESCE(cf.data_text, ci.normalized_text, '')
         FROM clipboard_items ci
         LEFT JOIN clipboard_formats cf ON cf.item_id = ci.id
              AND cf.data_kind IN ('text', 'html')
              AND COALESCE(cf.data_text, '') <> ''
         WHERE ci.id = ?1
           AND ci.is_deleted = 0
           AND ci.deleted_at IS NULL
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

fn load_item_image_path(
    connection: &rusqlite::Connection,
    id: &str,
) -> Result<String, CliplyError> {
    let result = connection.query_row(
        "SELECT data_path
         FROM clipboard_formats
         WHERE item_id = ?1
           AND data_kind = 'image_file'
           AND format_name <> 'thumbnail/png'
           AND COALESCE(data_path, '') <> ''
         ORDER BY priority DESC, created_at ASC
         LIMIT 1",
        params![id],
        |row| row.get::<_, String>(0),
    );

    match result {
        Ok(value) if !value.is_empty() => Ok(value),
        Ok(_) | Err(rusqlite::Error::QueryReturnedNoRows) => Err(CliplyError::PlatformUnavailable(
            "selected clipboard item has no image file".into(),
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
