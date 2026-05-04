use crate::error::CliplyError;
use crate::models::clipboard_item::{
    ClipboardFormatDto, ClipboardItemDetailDto, ClipboardItemDto, ClipboardItemType,
};
use crate::services::database_service;
use rusqlite::{params, Connection};
use tauri::AppHandle;

pub fn list_clipboard_items(
    app: &AppHandle,
    query: Option<String>,
    item_type: Option<String>,
    pinned_only: Option<bool>,
    limit: Option<i64>,
    offset: Option<i64>,
) -> Result<Vec<ClipboardItemDto>, CliplyError> {
    let connection = database_service::connect(app)?;
    let normalized_query = query.unwrap_or_default().trim().to_lowercase();
    let item_type = item_type.unwrap_or_default().to_lowercase();
    let pinned_only = pinned_only.unwrap_or(false);
    let limit = limit.unwrap_or(50).max(1) as usize;
    let offset = offset.unwrap_or(0).max(0) as usize;

    let mut items = load_items(&connection)?;

    if !normalized_query.is_empty() {
        items.retain(|item| {
            let haystack = format!(
                "{} {} {} {} {}",
                item.title,
                item.preview_text,
                item.source_app,
                item.source_window.clone().unwrap_or_default(),
                item.tags.join(" ")
            )
            .to_lowercase();
            haystack.contains(&normalized_query)
        });
    }

    if !item_type.is_empty() {
        items.retain(|item| matches_item_type(item, &item_type));
    }

    if pinned_only {
        items.retain(|item| item.is_pinned);
    }

    Ok(items.into_iter().skip(offset).take(limit).collect())
}

pub fn get_clipboard_item_detail(
    app: &AppHandle,
    id: String,
) -> Result<ClipboardItemDetailDto, CliplyError> {
    let connection = database_service::connect(app)?;
    let item = load_item(&connection, &id)?;
    let formats = load_formats(&connection, &id)?;
    let full_text = load_full_text(&connection, &id)?;

    Ok(ClipboardItemDetailDto {
        item,
        full_text,
        thumbnail_path: None,
        formats,
    })
}

pub fn toggle_pin_clipboard_item(
    app: &AppHandle,
    id: String,
) -> Result<ClipboardItemDto, CliplyError> {
    let connection = database_service::connect(app)?;
    connection.execute(
        "UPDATE clipboard_items
         SET is_pinned = CASE WHEN is_pinned = 1 THEN 0 ELSE 1 END,
             updated_at = datetime('now')
         WHERE id = ?1",
        params![id],
    )?;

    load_item(&connection, &id)
}

fn load_items(connection: &Connection) -> Result<Vec<ClipboardItemDto>, CliplyError> {
    let mut statement = connection.prepare(
        "SELECT id, type, COALESCE(title, ''), COALESCE(preview_text, ''),
                COALESCE(source_app, ''), source_window, copied_at, created_at,
                COALESCE(size_bytes, 0), is_pinned
         FROM clipboard_items
         WHERE is_deleted = 0
         ORDER BY is_pinned DESC, copied_at DESC",
    )?;

    let rows = statement.query_map([], |row| {
        let id: String = row.get(0)?;
        let item_type: String = row.get(1)?;
        Ok(ClipboardItemDto {
            tags: Vec::new(),
            id,
            item_type: parse_item_type(&item_type),
            title: row.get(2)?,
            preview_text: row.get(3)?,
            source_app: row.get(4)?,
            source_window: row.get(5)?,
            copied_at: row.get(6)?,
            created_at: row.get(7)?,
            relative_time: String::new(),
            size_bytes: row.get(8)?,
            is_pinned: row.get::<_, i64>(9)? == 1,
        })
    })?;

    let mut items = Vec::new();
    for row in rows {
        let mut item = row?;
        item.tags = load_tags(connection, &item.id)?;
        items.push(item);
    }

    Ok(items)
}

fn load_item(connection: &Connection, id: &str) -> Result<ClipboardItemDto, CliplyError> {
    let mut item = connection.query_row(
        "SELECT id, type, COALESCE(title, ''), COALESCE(preview_text, ''),
                COALESCE(source_app, ''), source_window, copied_at, created_at,
                COALESCE(size_bytes, 0), is_pinned
         FROM clipboard_items
         WHERE id = ?1 AND is_deleted = 0",
        params![id],
        |row| {
            let item_type: String = row.get(1)?;
            Ok(ClipboardItemDto {
                id: row.get(0)?,
                item_type: parse_item_type(&item_type),
                title: row.get(2)?,
                preview_text: row.get(3)?,
                source_app: row.get(4)?,
                source_window: row.get(5)?,
                copied_at: row.get(6)?,
                created_at: row.get(7)?,
                relative_time: String::new(),
                size_bytes: row.get(8)?,
                is_pinned: row.get::<_, i64>(9)? == 1,
                tags: Vec::new(),
            })
        },
    )?;
    item.tags = load_tags(connection, &item.id)?;
    Ok(item)
}

fn load_formats(
    connection: &Connection,
    item_id: &str,
) -> Result<Vec<ClipboardFormatDto>, CliplyError> {
    let mut statement = connection.prepare(
        "SELECT format_name, mime_type, data_kind, COALESCE(size_bytes, 0)
         FROM clipboard_formats
         WHERE item_id = ?1
         ORDER BY priority DESC, created_at ASC",
    )?;

    let rows = statement.query_map(params![item_id], |row| {
        Ok(ClipboardFormatDto {
            format_name: row.get(0)?,
            mime_type: row.get(1)?,
            data_kind: row.get(2)?,
            size_bytes: row.get(3)?,
        })
    })?;

    let mut formats = Vec::new();
    for row in rows {
        formats.push(row?);
    }

    Ok(formats)
}

fn load_full_text(connection: &Connection, item_id: &str) -> Result<Option<String>, CliplyError> {
    let mut statement = connection.prepare(
        "SELECT data_text
         FROM clipboard_formats
         WHERE item_id = ?1 AND data_kind IN ('text', 'html')
         ORDER BY priority DESC, created_at ASC
         LIMIT 1",
    )?;

    let result = statement.query_row(params![item_id], |row| row.get(0));
    match result {
        Ok(value) => Ok(value),
        Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
        Err(error) => Err(error.into()),
    }
}

fn load_tags(connection: &Connection, item_id: &str) -> Result<Vec<String>, CliplyError> {
    let mut statement = connection.prepare(
        "SELECT tag FROM clipboard_tags WHERE item_id = ?1 ORDER BY tag ASC",
    )?;
    let rows = statement.query_map(params![item_id], |row| row.get(0))?;

    let mut tags = Vec::new();
    for row in rows {
        tags.push(row?);
    }

    Ok(tags)
}

fn parse_item_type(value: &str) -> ClipboardItemType {
    match value {
        "link" => ClipboardItemType::Link,
        "image" => ClipboardItemType::Image,
        "code" => ClipboardItemType::Code,
        _ => ClipboardItemType::Text,
    }
}

fn matches_item_type(item: &ClipboardItemDto, expected: &str) -> bool {
    matches!(
        (&item.item_type, expected),
        (ClipboardItemType::Text, "text")
            | (ClipboardItemType::Link, "link")
            | (ClipboardItemType::Image, "image")
            | (ClipboardItemType::Code, "code")
    )
}
