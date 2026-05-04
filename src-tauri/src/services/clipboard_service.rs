use crate::error::CliplyError;
use crate::models::clipboard_item::{
    ClipboardFormatDto, ClipboardItemDetailDto, ClipboardItemDto, ClipboardItemType,
};
use crate::platform::{self, ClipboardSnapshot};
use crate::services::{database_service, hash_service, sensitive_detector};
use rusqlite::{params, Connection};
use tauri::AppHandle;
use time::OffsetDateTime;
use uuid::Uuid;

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

pub fn ingest_current_clipboard(app: &AppHandle) -> Result<Option<ClipboardItemDto>, CliplyError> {
    let snapshot = match platform::read_current_clipboard()? {
        Some(snapshot) => snapshot,
        None => return Ok(None),
    };

    ingest_clipboard_snapshot(app, snapshot)
}

pub fn ingest_clipboard_snapshot(
    app: &AppHandle,
    snapshot: ClipboardSnapshot,
) -> Result<Option<ClipboardItemDto>, CliplyError> {
    let text = match snapshot.text.as_deref() {
        Some(text) if !text.trim().is_empty() => text,
        _ => return Ok(None),
    };

    if sensitive_detector::looks_sensitive(text) {
        return Ok(None);
    }

    let connection = database_service::connect(app)?;
    let normalized_text = hash_service::normalize_text_for_hash(text);
    let hash = hash_service::stable_text_hash(text);
    let now = OffsetDateTime::now_utc()
        .format(&time::format_description::well_known::Rfc3339)
        .map_err(|error| CliplyError::StorageUnavailable(error.to_string()))?;

    if let Some(existing_id) = find_existing_item_id(&connection, &hash)? {
        connection.execute(
            "UPDATE clipboard_items
             SET copied_at = ?1,
                 updated_at = ?1,
                 used_count = COALESCE(used_count, 0) + 1
             WHERE id = ?2 AND is_deleted = 0",
            params![now, existing_id],
        )?;

        return Ok(Some(load_item(&connection, &existing_id)?));
    }

    let id = Uuid::new_v4().to_string();
    let item_type = item_type_as_str(&snapshot.primary_type);
    let title = title_from_text(text);
    let preview_text = preview_from_text(text);
    let source_app = snapshot
        .source_app
        .unwrap_or_else(|| "Windows Clipboard".into());
    let source_window = snapshot.source_window;
    let size_bytes = text.len() as i64;

    connection.execute(
        "INSERT INTO clipboard_items (
            id, type, title, preview_text, normalized_text, source_app, source_window,
            hash, size_bytes, is_pinned, copied_at, created_at, updated_at, used_count
        ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, 0, ?10, ?10, ?10, 0)",
        params![
            id,
            item_type,
            title,
            preview_text,
            normalized_text,
            source_app,
            source_window,
            hash,
            size_bytes,
            now
        ],
    )?;

    connection.execute(
        "INSERT INTO clipboard_formats (
            id, item_id, format_name, mime_type, data_kind, data_text, size_bytes, priority, created_at
        ) VALUES (?1, ?2, 'text/plain', 'text/plain', 'text', ?3, ?4, 100, ?5)",
        params![
            format!("{id}-format-text"),
            id,
            text,
            size_bytes,
            now
        ],
    )?;

    for (index, format) in snapshot.formats.iter().enumerate() {
        connection.execute(
            "INSERT OR IGNORE INTO clipboard_formats (
                id, item_id, format_name, mime_type, data_kind, data_text, size_bytes, priority, created_at
            ) VALUES (?1, ?2, ?3, ?4, ?5, NULL, 0, ?6, ?7)",
            params![
                format!("{id}-format-{index}"),
                id,
                format.format_name,
                format.mime_type,
                format.data_kind,
                50 - index as i64,
                now
            ],
        )?;
    }

    upsert_fts(
        &connection,
        &id,
        &title,
        &preview_text,
        &normalized_text,
        &source_app,
    )?;

    Ok(Some(load_item(&connection, &id)?))
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
    let mut statement =
        connection.prepare("SELECT tag FROM clipboard_tags WHERE item_id = ?1 ORDER BY tag ASC")?;
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

fn find_existing_item_id(
    connection: &Connection,
    hash: &str,
) -> Result<Option<String>, CliplyError> {
    let result = connection.query_row(
        "SELECT id FROM clipboard_items WHERE hash = ?1 AND is_deleted = 0 LIMIT 1",
        params![hash],
        |row| row.get(0),
    );

    match result {
        Ok(id) => Ok(Some(id)),
        Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
        Err(error) => Err(error.into()),
    }
}

fn upsert_fts(
    connection: &Connection,
    item_id: &str,
    title: &str,
    preview_text: &str,
    normalized_text: &str,
    source_app: &str,
) -> Result<(), CliplyError> {
    connection.execute(
        "DELETE FROM clipboard_items_fts WHERE item_id = ?1",
        params![item_id],
    )?;
    connection.execute(
        "INSERT INTO clipboard_items_fts (
            item_id, title, preview_text, normalized_text, source_app
        ) VALUES (?1, ?2, ?3, ?4, ?5)",
        params![item_id, title, preview_text, normalized_text, source_app],
    )?;

    Ok(())
}

fn title_from_text(value: &str) -> String {
    preview_from_text(value)
}

fn preview_from_text(value: &str) -> String {
    let preview = value
        .lines()
        .map(str::trim)
        .find(|line| !line.is_empty())
        .unwrap_or("")
        .chars()
        .take(140)
        .collect::<String>();

    if preview.is_empty() {
        "Untitled text".into()
    } else {
        preview
    }
}

fn item_type_as_str(item_type: &ClipboardItemType) -> &'static str {
    match item_type {
        ClipboardItemType::Text => "text",
        ClipboardItemType::Link => "link",
        ClipboardItemType::Image => "image",
        ClipboardItemType::Code => "code",
    }
}
