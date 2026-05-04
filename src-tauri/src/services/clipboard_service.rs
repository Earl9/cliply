use crate::error::CliplyError;
use crate::models::clipboard_item::{
    ClipboardFormatDto, ClipboardItemDetailDto, ClipboardItemDto, ClipboardItemType,
};
use crate::models::settings::CliplySettings;
use crate::platform::{self, ClipboardSnapshot};
use crate::services::{
    blob_service, database_service, hash_service, sensitive_detector, settings_service,
};
use rusqlite::{params, Connection, Row};
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
    let normalized_query =
        crate::services::search_service::normalize_query(&query.unwrap_or_default());
    let item_type = item_type.unwrap_or_default().to_lowercase();
    let pinned_only = pinned_only.unwrap_or(false);
    let limit = limit.unwrap_or(50).max(1);
    let offset = offset.unwrap_or(0).max(0);

    if normalized_query.is_empty() {
        load_items(&connection, &item_type, pinned_only, limit, offset)
    } else {
        search_items(
            &connection,
            &normalized_query,
            &item_type,
            pinned_only,
            limit,
            offset,
        )
    }
}

pub fn get_clipboard_item_detail(
    app: &AppHandle,
    id: String,
) -> Result<ClipboardItemDetailDto, CliplyError> {
    let connection = database_service::connect(app)?;
    let item = load_item(&connection, &id)?;
    let formats = load_formats(&connection, &id)?;
    let full_text = load_full_text(&connection, &id)?;
    let thumbnail_path = load_image_path(&connection, &id, true)?;
    let image_path = load_image_path(&connection, &id, false)?;
    let (image_width, image_height) = image_dimensions_from_preview(&item.preview_text);

    Ok(ClipboardItemDetailDto {
        item,
        full_text,
        thumbnail_path,
        image_path,
        image_width,
        image_height,
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

pub fn delete_clipboard_item(app: &AppHandle, id: String) -> Result<(), CliplyError> {
    let mut connection = database_service::connect(app)?;
    let transaction = connection.transaction()?;

    transaction.execute(
        "UPDATE clipboard_items
         SET is_deleted = 1,
             updated_at = datetime('now')
         WHERE id = ?1",
        params![id],
    )?;
    transaction.execute(
        "DELETE FROM clipboard_items_fts WHERE item_id = ?1",
        params![id],
    )?;

    transaction.commit()?;
    Ok(())
}

pub fn clear_clipboard_history(app: &AppHandle, include_pinned: bool) -> Result<(), CliplyError> {
    let mut connection = database_service::connect(app)?;
    let transaction = connection.transaction()?;
    let include_pinned_flag = if include_pinned { 1 } else { 0 };

    let item_ids = {
        let mut statement = transaction.prepare(
            "SELECT id
             FROM clipboard_items
             WHERE is_deleted = 0
               AND (?1 = 1 OR is_pinned = 0)",
        )?;
        let rows =
            statement.query_map(params![include_pinned_flag], |row| row.get::<_, String>(0))?;
        let mut item_ids = Vec::new();
        for row in rows {
            item_ids.push(row?);
        }
        item_ids
    };

    transaction.execute(
        "UPDATE clipboard_items
         SET is_deleted = 1,
             updated_at = datetime('now')
         WHERE is_deleted = 0
           AND (?1 = 1 OR is_pinned = 0)",
        params![include_pinned_flag],
    )?;

    for item_id in item_ids {
        transaction.execute(
            "DELETE FROM clipboard_items_fts WHERE item_id = ?1",
            params![item_id],
        )?;
    }

    transaction.commit()?;
    Ok(())
}

pub fn ingest_current_clipboard(app: &AppHandle) -> Result<Option<ClipboardItemDto>, CliplyError> {
    if settings_service::is_monitoring_paused(app) {
        return Ok(None);
    }

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
    let settings = settings_service::get_settings(app)
        .unwrap_or_else(|_| settings_service::default_settings());
    let source_app = source_app_from_snapshot(&snapshot);

    if is_ignored_source_app(&source_app, &settings.ignore_apps) {
        return Ok(None);
    }

    let has_text = snapshot
        .text
        .as_deref()
        .is_some_and(|text| !text.trim().is_empty());
    if !has_text && snapshot.image.is_some() {
        return ingest_image_snapshot(app, snapshot, &settings);
    }

    let text = match snapshot.text.as_deref() {
        Some(text) if !text.trim().is_empty() => text,
        _ => return Ok(None),
    };

    let sensitivity = sensitive_detector::analyze(text);
    if !settings.save_sensitive && sensitivity.risk == sensitive_detector::SensitivityRisk::High {
        return Ok(None);
    }
    let redact_sensitive_text =
        !settings.save_sensitive && sensitivity.risk == sensitive_detector::SensitivityRisk::Medium;

    let connection = database_service::connect(app)?;
    let normalized_text = if redact_sensitive_text {
        String::new()
    } else {
        hash_service::normalize_text_for_hash(text)
    };
    let hash = if redact_sensitive_text {
        format!("redacted-{}", Uuid::new_v4())
    } else {
        hash_service::stable_text_hash(text)
    };
    let now = OffsetDateTime::now_utc()
        .format(&time::format_description::well_known::Rfc3339)
        .map_err(|error| CliplyError::StorageUnavailable(error.to_string()))?;

    if !redact_sensitive_text && settings.ignore_duplicate {
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
    }

    let id = Uuid::new_v4().to_string();
    let item_type = item_type_as_str(&snapshot.primary_type);
    let (title, preview_text) = if redact_sensitive_text {
        (
            "已隐藏敏感内容".to_string(),
            "已隐藏疑似验证码等敏感内容".to_string(),
        )
    } else {
        (title_from_text(text), preview_from_text(text))
    };
    let source_window = snapshot.source_window;
    let size_bytes = text.len() as i64;
    let sensitive_score = i64::from(sensitivity.score);

    connection.execute(
        "INSERT INTO clipboard_items (
            id, type, title, preview_text, normalized_text, source_app, source_window,
            hash, size_bytes, is_pinned, sensitive_score, copied_at, created_at, updated_at, used_count
        ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, 0, ?10, ?11, ?11, ?11, 0)",
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
            sensitive_score,
            now
        ],
    )?;

    if !redact_sensitive_text {
        connection.execute(
            "INSERT INTO clipboard_formats (
                id, item_id, format_name, mime_type, data_kind, data_text, size_bytes, priority, created_at
            ) VALUES (?1, ?2, 'text/plain', 'text/plain', 'text', ?3, ?4, 100, ?5)",
            params![format!("{id}-format-text"), id, text, size_bytes, now],
        )?;
    }

    for (index, format) in snapshot.formats.iter().enumerate() {
        let data_kind =
            if redact_sensitive_text && matches!(format.data_kind.as_str(), "text" | "html") {
                "external_ref"
            } else {
                format.data_kind.as_str()
            };

        connection.execute(
            "INSERT OR IGNORE INTO clipboard_formats (
                id, item_id, format_name, mime_type, data_kind, data_text, size_bytes, priority, created_at
            ) VALUES (?1, ?2, ?3, ?4, ?5, NULL, 0, ?6, ?7)",
            params![
                format!("{id}-format-{index}"),
                id,
                format.format_name,
                format.mime_type,
                data_kind,
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

fn ingest_image_snapshot(
    app: &AppHandle,
    snapshot: ClipboardSnapshot,
    settings: &CliplySettings,
) -> Result<Option<ClipboardItemDto>, CliplyError> {
    if !settings.save_images {
        return Ok(None);
    }

    let source_app = source_app_from_snapshot(&snapshot);
    let source_window = snapshot.source_window.clone();
    let Some(image) = snapshot.image else {
        return Ok(None);
    };

    if image.bytes.is_empty() {
        return Ok(None);
    }

    let connection = database_service::connect(app)?;
    let hash = hash_service::stable_bytes_hash(&image.bytes);
    let now = OffsetDateTime::now_utc()
        .format(&time::format_description::well_known::Rfc3339)
        .map_err(|error| CliplyError::StorageUnavailable(error.to_string()))?;

    if settings.ignore_duplicate {
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
    }

    let id = Uuid::new_v4().to_string();
    let stored_blob = blob_service::store_image(app, &id, &image)?;
    let title = format!("图片 {} x {}", image.width, image.height);
    let preview_text = format!(
        "图片 {} x {} {}",
        image.width,
        image.height,
        image.extension.to_uppercase()
    );

    connection.execute(
        "INSERT INTO clipboard_items (
            id, type, title, preview_text, normalized_text, source_app, source_window,
            hash, size_bytes, is_pinned, sensitive_score, copied_at, created_at, updated_at, used_count
        ) VALUES (?1, 'image', ?2, ?3, '', ?4, ?5, ?6, ?7, 0, 0, ?8, ?8, ?8, 0)",
        params![
            id,
            title,
            preview_text,
            source_app,
            source_window,
            hash,
            stored_blob.size_bytes,
            now
        ],
    )?;

    connection.execute(
        "INSERT INTO clipboard_formats (
            id, item_id, format_name, mime_type, data_kind, data_path, size_bytes, priority, created_at
        ) VALUES (?1, ?2, ?3, ?4, 'image_file', ?5, ?6, 100, ?7)",
        params![
            format!("{id}-format-image"),
            id,
            format!("image/{}", image.extension),
            image.mime_type,
            stored_blob.image_path.to_string_lossy().to_string(),
            stored_blob.size_bytes,
            now
        ],
    )?;

    let thumbnail_size = std::fs::metadata(&stored_blob.thumbnail_path)
        .map(|metadata| metadata.len() as i64)
        .unwrap_or(0);
    connection.execute(
        "INSERT INTO clipboard_formats (
            id, item_id, format_name, mime_type, data_kind, data_path, size_bytes, priority, created_at
        ) VALUES (?1, ?2, 'thumbnail/png', 'image/png', 'image_file', ?3, ?4, 90, ?5)",
        params![
            format!("{id}-format-thumbnail"),
            id,
            stored_blob.thumbnail_path.to_string_lossy().to_string(),
            thumbnail_size,
            now
        ],
    )?;

    for (index, format) in snapshot.formats.iter().enumerate() {
        connection.execute(
            "INSERT OR IGNORE INTO clipboard_formats (
                id, item_id, format_name, mime_type, data_kind, data_text, size_bytes, priority, created_at
            ) VALUES (?1, ?2, ?3, ?4, ?5, NULL, 0, ?6, ?7)",
            params![
                format!("{id}-format-raw-{index}"),
                id,
                format.format_name,
                format.mime_type,
                format.data_kind,
                40 - index as i64,
                now
            ],
        )?;
    }

    upsert_fts(&connection, &id, &title, &preview_text, "", &source_app)?;

    Ok(Some(load_item(&connection, &id)?))
}

fn load_items(
    connection: &Connection,
    item_type: &str,
    pinned_only: bool,
    limit: i64,
    offset: i64,
) -> Result<Vec<ClipboardItemDto>, CliplyError> {
    let mut statement = connection.prepare(
        "SELECT id, type, COALESCE(title, ''), COALESCE(preview_text, ''),
                COALESCE(source_app, ''), source_window, copied_at, created_at,
                COALESCE(size_bytes, 0), is_pinned, COALESCE(sensitive_score, 0)
         FROM clipboard_items
         WHERE is_deleted = 0
           AND (?1 = '' OR type = ?1)
           AND (?2 = 0 OR is_pinned = 1)
         ORDER BY is_pinned DESC, copied_at DESC
         LIMIT ?3 OFFSET ?4",
    )?;

    let pinned_only = if pinned_only { 1 } else { 0 };
    let rows = statement.query_map(params![item_type, pinned_only, limit, offset], map_item_row)?;

    collect_items(connection, rows)
}

fn load_item(connection: &Connection, id: &str) -> Result<ClipboardItemDto, CliplyError> {
    let mut item = connection.query_row(
        "SELECT id, type, COALESCE(title, ''), COALESCE(preview_text, ''),
                COALESCE(source_app, ''), source_window, copied_at, created_at,
                COALESCE(size_bytes, 0), is_pinned, COALESCE(sensitive_score, 0)
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
                sensitive_score: row.get(10)?,
                tags: Vec::new(),
                thumbnail_path: None,
            })
        },
    )?;
    item.tags = load_tags(connection, &item.id)?;
    item.thumbnail_path = load_image_path(connection, &item.id, true)?;
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

fn load_image_path(
    connection: &Connection,
    item_id: &str,
    thumbnail: bool,
) -> Result<Option<String>, CliplyError> {
    let result = if thumbnail {
        connection.query_row(
            "SELECT data_path
             FROM clipboard_formats
             WHERE item_id = ?1
               AND data_kind = 'image_file'
               AND format_name = 'thumbnail/png'
               AND data_path IS NOT NULL
             ORDER BY priority DESC, created_at ASC
             LIMIT 1",
            params![item_id],
            |row| row.get::<_, String>(0),
        )
    } else {
        connection.query_row(
            "SELECT data_path
             FROM clipboard_formats
             WHERE item_id = ?1
               AND data_kind = 'image_file'
               AND format_name <> 'thumbnail/png'
               AND data_path IS NOT NULL
             ORDER BY priority DESC, created_at ASC
             LIMIT 1",
            params![item_id],
            |row| row.get::<_, String>(0),
        )
    };

    match result {
        Ok(value) => Ok(Some(value)),
        Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
        Err(error) => Err(error.into()),
    }
}

fn image_dimensions_from_preview(preview_text: &str) -> (Option<u32>, Option<u32>) {
    let parts = preview_text.split_whitespace().collect::<Vec<_>>();
    if parts.len() < 4 || !matches!(parts[0], "Image" | "图片") || !matches!(parts[2], "x" | "×")
    {
        return (None, None);
    }

    let width = parts[1].parse::<u32>().ok();
    let height = parts[3].parse::<u32>().ok();
    (width, height)
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

fn search_items(
    connection: &Connection,
    query: &str,
    item_type: &str,
    pinned_only: bool,
    limit: i64,
    offset: i64,
) -> Result<Vec<ClipboardItemDto>, CliplyError> {
    let pinned_only = if pinned_only { 1 } else { 0 };
    let like_query = format!("%{}%", escape_like(query));

    if let Some(fts_query) = build_fts_query(query) {
        let mut statement = connection.prepare(
            "SELECT DISTINCT ci.id, ci.type, COALESCE(ci.title, ''),
                    COALESCE(ci.preview_text, ''), COALESCE(ci.source_app, ''),
                    ci.source_window, ci.copied_at, ci.created_at,
                    COALESCE(ci.size_bytes, 0), ci.is_pinned, COALESCE(ci.sensitive_score, 0)
             FROM clipboard_items ci
             WHERE ci.is_deleted = 0
               AND (?3 = '' OR ci.type = ?3)
               AND (?4 = 0 OR ci.is_pinned = 1)
               AND (
                    ci.id IN (
                        SELECT item_id
                        FROM clipboard_items_fts
                        WHERE clipboard_items_fts MATCH ?1
                    )
                    OR lower(COALESCE(ci.title, '')) LIKE ?2 ESCAPE '\\'
                    OR lower(COALESCE(ci.preview_text, '')) LIKE ?2 ESCAPE '\\'
                    OR lower(COALESCE(ci.normalized_text, '')) LIKE ?2 ESCAPE '\\'
                    OR lower(COALESCE(ci.source_app, '')) LIKE ?2 ESCAPE '\\'
                    OR lower(COALESCE(ci.source_window, '')) LIKE ?2 ESCAPE '\\'
                    OR EXISTS (
                        SELECT 1
                        FROM clipboard_tags tag
                        WHERE tag.item_id = ci.id
                          AND lower(tag.tag) LIKE ?2 ESCAPE '\\'
                    )
               )
             ORDER BY ci.is_pinned DESC, ci.copied_at DESC
             LIMIT ?5 OFFSET ?6",
        )?;

        let rows = statement.query_map(
            params![fts_query, like_query, item_type, pinned_only, limit, offset],
            map_item_row,
        )?;

        return collect_items(connection, rows);
    }

    let mut statement = connection.prepare(
        "SELECT DISTINCT ci.id, ci.type, COALESCE(ci.title, ''),
                COALESCE(ci.preview_text, ''), COALESCE(ci.source_app, ''),
                ci.source_window, ci.copied_at, ci.created_at,
                COALESCE(ci.size_bytes, 0), ci.is_pinned, COALESCE(ci.sensitive_score, 0)
         FROM clipboard_items ci
         WHERE ci.is_deleted = 0
           AND (?2 = '' OR ci.type = ?2)
           AND (?3 = 0 OR ci.is_pinned = 1)
           AND (
                lower(COALESCE(ci.title, '')) LIKE ?1 ESCAPE '\\'
                OR lower(COALESCE(ci.preview_text, '')) LIKE ?1 ESCAPE '\\'
                OR lower(COALESCE(ci.normalized_text, '')) LIKE ?1 ESCAPE '\\'
                OR lower(COALESCE(ci.source_app, '')) LIKE ?1 ESCAPE '\\'
                OR lower(COALESCE(ci.source_window, '')) LIKE ?1 ESCAPE '\\'
                OR EXISTS (
                    SELECT 1
                    FROM clipboard_tags tag
                    WHERE tag.item_id = ci.id
                      AND lower(tag.tag) LIKE ?1 ESCAPE '\\'
                )
           )
         ORDER BY ci.is_pinned DESC, ci.copied_at DESC
         LIMIT ?4 OFFSET ?5",
    )?;

    let rows = statement.query_map(
        params![like_query, item_type, pinned_only, limit, offset],
        map_item_row,
    )?;

    collect_items(connection, rows)
}

fn collect_items<I>(connection: &Connection, rows: I) -> Result<Vec<ClipboardItemDto>, CliplyError>
where
    I: IntoIterator<Item = rusqlite::Result<ClipboardItemDto>>,
{
    let mut items = Vec::new();
    for row in rows {
        let mut item = row?;
        item.tags = load_tags(connection, &item.id)?;
        item.thumbnail_path = load_image_path(connection, &item.id, true)?;
        items.push(item);
    }

    Ok(items)
}

fn source_app_from_snapshot(snapshot: &ClipboardSnapshot) -> String {
    snapshot
        .source_app
        .as_deref()
        .map(str::trim)
        .filter(|source_app| !source_app.is_empty())
        .unwrap_or("Windows Clipboard")
        .to_string()
}

fn is_ignored_source_app(source_app: &str, ignore_apps: &[String]) -> bool {
    let normalized_source = normalize_app_name(source_app);

    ignore_apps
        .iter()
        .map(|app| normalize_app_name(app))
        .any(|ignored_app| {
            !ignored_app.is_empty()
                && (normalized_source == ignored_app
                    || normalized_source.contains(&ignored_app)
                    || ignored_app.contains(&normalized_source))
        })
}

fn normalize_app_name(value: &str) -> String {
    value
        .trim()
        .trim_end_matches(".exe")
        .to_lowercase()
        .chars()
        .filter(|character| !character.is_whitespace() && *character != '-' && *character != '_')
        .collect()
}

fn map_item_row(row: &Row<'_>) -> rusqlite::Result<ClipboardItemDto> {
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
        sensitive_score: row.get(10)?,
        thumbnail_path: None,
    })
}

fn build_fts_query(query: &str) -> Option<String> {
    let terms = query
        .split_whitespace()
        .filter_map(|term| {
            let sanitized = term
                .chars()
                .filter(|character| character.is_alphanumeric() || *character == '_')
                .collect::<String>();
            if sanitized.is_empty() {
                None
            } else {
                Some(format!("\"{}\"", sanitized.replace('"', "\"\"")))
            }
        })
        .collect::<Vec<_>>();

    if terms.is_empty() {
        None
    } else {
        Some(terms.join(" AND "))
    }
}

fn escape_like(value: &str) -> String {
    value
        .replace('\\', "\\\\")
        .replace('%', "\\%")
        .replace('_', "\\_")
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

#[cfg(test)]
mod tests {
    use super::{is_ignored_source_app, normalize_app_name};

    #[test]
    fn matches_ignored_apps_case_and_exe_insensitively() {
        let ignored = vec!["KeePassXC".to_string(), "1Password".to_string()];

        assert!(is_ignored_source_app("keepassxc.exe", &ignored));
        assert!(is_ignored_source_app("1password", &ignored));
    }

    #[test]
    fn does_not_match_unrelated_apps() {
        let ignored = vec!["Bitwarden".to_string()];

        assert!(!is_ignored_source_app("Visual Studio Code", &ignored));
    }

    #[test]
    fn normalizes_app_names_for_matching() {
        assert_eq!(normalize_app_name("KeePass-XC.exe"), "keepassxc");
    }
}
