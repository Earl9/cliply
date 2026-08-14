use crate::error::CliplyError;
use crate::services::{
    database_service, sync_crypto_service, sync_merge_service, sync_queue_service, sync_service,
};
use rusqlite::{params, Connection, OptionalExtension};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use tauri::AppHandle;
use time::OffsetDateTime;

pub const SYNC_PACKAGE_APP: &str = "Cliply";
pub const SYNC_PACKAGE_VERSION: u32 = 1;
pub const SYNC_PACKAGE_EXTENSION: &str = "cliply-sync";

const LAST_EXPORTED_AT_KEY: &str = "last_sync_package_exported_at";
const LAST_IMPORTED_AT_KEY: &str = "last_sync_package_imported_at";

// Tombstones and already-synced events only exist to propagate state to other
// devices; after this window every device is assumed to have seen them, so
// exports stop carrying them and the payload stops growing with history.
pub const SYNC_EXPORT_WINDOW_DAYS: i64 = 30;

#[derive(Debug)]
pub struct BuiltSyncPackage {
    pub bytes: Vec<u8>,
    pub exported_at: String,
    pub queue_checkpoint: sync_queue_service::SyncExportCheckpoint,
}

fn export_cutoff_timestamp() -> Result<String, CliplyError> {
    (OffsetDateTime::now_utc() - time::Duration::days(SYNC_EXPORT_WINDOW_DAYS))
        .format(&time::format_description::well_known::Rfc3339)
        .map_err(|error| CliplyError::StorageUnavailable(error.to_string()))
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct SyncPackageEnvelope {
    pub version: u32,
    pub app: String,
    pub exported_at: String,
    pub device: SyncPackageDevice,
    pub encryption: sync_crypto_service::SyncEncryptionMetadata,
    pub encrypted_payload: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub struct SyncPackageDevice {
    pub id: String,
    pub name: String,
    pub platform: String,
    pub created_at: String,
    pub last_seen_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct SyncPackagePayload {
    pub version: u32,
    pub app: String,
    pub exported_at: String,
    pub device: SyncPackageDevice,
    pub items: Vec<SyncPackageItem>,
    #[serde(default)]
    pub sync_blobs: Vec<SyncPackageBlob>,
    pub sync_events: Vec<SyncPackageEvent>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct SyncPackageItem {
    pub id: String,
    pub item_type: String,
    pub title: Option<String>,
    pub preview_text: Option<String>,
    pub normalized_text: Option<String>,
    pub source_app: Option<String>,
    pub source_window: Option<String>,
    pub hash: String,
    pub size_bytes: i64,
    pub is_pinned: bool,
    pub sensitive_score: i64,
    pub copied_at: String,
    pub created_at: String,
    pub updated_at: String,
    pub used_count: i64,
    pub sync_id: String,
    pub device_id: String,
    pub revision: i64,
    pub deleted_at: Option<String>,
    pub sync_status: Option<String>,
    pub last_synced_at: Option<String>,
    pub formats: Vec<SyncPackageFormat>,
    pub tags: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct SyncPackageFormat {
    pub format_name: String,
    pub mime_type: Option<String>,
    pub data_kind: String,
    pub data_text: Option<String>,
    pub size_bytes: i64,
    pub priority: i64,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct SyncPackageBlob {
    pub id: String,
    pub item_id: String,
    pub blob_type: String,
    pub remote_path: Option<String>,
    pub size_bytes: i64,
    pub hash: String,
    pub encrypted: bool,
    pub sync_status: Option<String>,
    pub created_at: String,
    pub uploaded_at: Option<String>,
    pub deleted_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct SyncPackageEvent {
    pub id: String,
    pub item_id: Option<String>,
    pub event_type: String,
    pub payload_json: Option<String>,
    pub created_at: String,
    pub synced_at: Option<String>,
}

#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct SyncImportResult {
    pub imported_count: usize,
    pub updated_count: usize,
    pub skipped_count: usize,
    pub deleted_count: usize,
    pub conflicted_count: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SyncPackageStatus {
    pub last_exported_at: Option<String>,
    pub last_imported_at: Option<String>,
}

pub fn export_sync_package(
    app: &AppHandle,
    path: String,
    password: String,
) -> Result<(), CliplyError> {
    let package = build_sync_package_bytes(app, &password)?;
    let connection = database_service::connect(app)?;
    let output_path = normalize_package_path(Path::new(&path));
    if let Some(parent) = output_path.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(output_path, package.bytes)?;
    set_sync_state_value(&connection, LAST_EXPORTED_AT_KEY, &package.exported_at)?;
    Ok(())
}

pub fn build_sync_package_bytes(
    app: &AppHandle,
    password: &str,
) -> Result<BuiltSyncPackage, CliplyError> {
    let mut connection = database_service::connect(app)?;
    let exported_at = current_timestamp()?;
    let transaction = connection.transaction()?;
    let queue_checkpoint = sync_queue_service::load_sync_export_checkpoint(&transaction)?;
    let device = load_current_device(&transaction)?;
    let payload = SyncPackagePayload {
        version: SYNC_PACKAGE_VERSION,
        app: SYNC_PACKAGE_APP.to_string(),
        exported_at: exported_at.clone(),
        device: device.clone(),
        items: load_export_items(&transaction, &exported_at)?,
        sync_blobs: load_export_sync_blobs(&transaction, &exported_at)?,
        sync_events: load_export_events(&transaction, &exported_at)?,
    };
    transaction.commit()?;
    let payload_json = serde_json::to_vec(&payload)
        .map_err(|error| CliplyError::Sync(format!("同步包序列化失败: {error}")))?;
    let (encryption, encrypted_payload) =
        sync_crypto_service::encrypt_payload(&payload_json, &password)?;

    let envelope = SyncPackageEnvelope {
        version: SYNC_PACKAGE_VERSION,
        app: SYNC_PACKAGE_APP.to_string(),
        exported_at: exported_at.clone(),
        device,
        encryption,
        encrypted_payload,
    };
    let package_json = serde_json::to_vec_pretty(&envelope)
        .map_err(|error| CliplyError::Sync(format!("同步包序列化失败: {error}")))?;
    Ok(BuiltSyncPackage {
        bytes: package_json,
        exported_at,
        queue_checkpoint,
    })
}

pub fn import_sync_package(
    app: &AppHandle,
    path: String,
    password: String,
) -> Result<SyncImportResult, CliplyError> {
    let package_bytes = fs::read(path)?;
    import_sync_package_bytes(app, &package_bytes, &password)
}

pub fn import_sync_package_bytes(
    app: &AppHandle,
    package_bytes: &[u8],
    password: &str,
) -> Result<SyncImportResult, CliplyError> {
    let package_json = std::str::from_utf8(package_bytes)
        .map_err(|_| CliplyError::Sync("文件格式不正确".to_string()))?;
    let envelope: SyncPackageEnvelope = serde_json::from_str(package_json)
        .map_err(|_| CliplyError::Sync("文件格式不正确".to_string()))?;
    validate_envelope(&envelope)?;

    let payload_bytes = sync_crypto_service::decrypt_payload(
        &envelope.encryption,
        &envelope.encrypted_payload,
        &password,
    )?;
    let payload: SyncPackagePayload = serde_json::from_slice(&payload_bytes)
        .map_err(|_| CliplyError::Sync("密码错误或同步包已损坏".to_string()))?;
    validate_payload(&payload)?;

    import_sync_payload(app, &payload)
}

pub fn import_sync_payload(
    app: &AppHandle,
    payload: &SyncPackagePayload,
) -> Result<SyncImportResult, CliplyError> {
    let mut connection = database_service::connect(app)?;
    let imported_at = current_timestamp()?;
    let transaction = connection.transaction()?;
    let result = sync_merge_service::merge_sync_payload(&transaction, &payload)
        .and_then(|result| {
            set_sync_state_value(&transaction, LAST_IMPORTED_AT_KEY, &imported_at)?;
            Ok(result)
        })
        .map_err(|error| CliplyError::Sync(format!("导入失败，已回滚: {error}")))?;
    transaction
        .commit()
        .map_err(|error| CliplyError::Sync(format!("导入失败，已回滚: {error}")))?;
    Ok(result)
}

pub fn get_sync_package_status(app: &AppHandle) -> Result<SyncPackageStatus, CliplyError> {
    let connection = database_service::connect(app)?;
    Ok(SyncPackageStatus {
        last_exported_at: get_sync_state_value(&connection, LAST_EXPORTED_AT_KEY)?,
        last_imported_at: get_sync_state_value(&connection, LAST_IMPORTED_AT_KEY)?,
    })
}

fn validate_envelope(envelope: &SyncPackageEnvelope) -> Result<(), CliplyError> {
    if envelope.app != SYNC_PACKAGE_APP {
        return Err(CliplyError::Sync("文件格式不正确".to_string()));
    }
    if envelope.version != SYNC_PACKAGE_VERSION {
        return Err(CliplyError::Sync("同步包版本不兼容".to_string()));
    }
    Ok(())
}

fn validate_payload(payload: &SyncPackagePayload) -> Result<(), CliplyError> {
    if payload.app != SYNC_PACKAGE_APP {
        return Err(CliplyError::Sync("文件格式不正确".to_string()));
    }
    if payload.version != SYNC_PACKAGE_VERSION {
        return Err(CliplyError::Sync("同步包版本不兼容".to_string()));
    }
    Ok(())
}

fn load_current_device(connection: &Connection) -> Result<SyncPackageDevice, CliplyError> {
    let device_id = sync_service::current_device_id(connection)?;
    let device = connection
        .query_row(
            "SELECT id, name, platform, created_at, last_seen_at
             FROM devices
             WHERE id = ?1",
            params![device_id],
            |row| {
                Ok(SyncPackageDevice {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    platform: row.get(2)?,
                    created_at: row.get(3)?,
                    last_seen_at: row.get(4)?,
                })
            },
        )
        .optional()?;

    device.ok_or_else(|| CliplyError::Sync("当前设备尚未初始化".to_string()))
}

fn load_export_items(
    connection: &Connection,
    exported_at: &str,
) -> Result<Vec<SyncPackageItem>, CliplyError> {
    let cutoff = export_cutoff_timestamp()?;
    // Live items always export. Pending or abandoned tombstones remain in every
    // export until a remote snapshot succeeds; only old synced tombstones expire.
    let mut statement = connection.prepare(
        "SELECT id, type, title, preview_text, normalized_text, source_app, source_window,
                hash, COALESCE(size_bytes, 0), COALESCE(is_pinned, 0),
                COALESCE(sensitive_score, 0), copied_at, created_at, updated_at,
                COALESCE(used_count, 0), COALESCE(sync_id, id), COALESCE(device_id, ''),
                COALESCE(revision, 1), deleted_at, sync_status, last_synced_at
         FROM clipboard_items
         WHERE sync_id IS NOT NULL
           AND julianday(updated_at) <= julianday(?2)
           AND (
             deleted_at IS NULL
             OR COALESCE(sync_status, 'pending') <> 'synced'
             OR (
               sync_status = 'synced'
               AND julianday(deleted_at) >= julianday(?1)
             )
           )
         ORDER BY datetime(updated_at) ASC, id ASC",
    )?;
    let rows = statement.query_map(params![cutoff, exported_at], |row| {
        Ok(SyncPackageItem {
            id: row.get(0)?,
            item_type: row.get(1)?,
            title: row.get(2)?,
            preview_text: row.get(3)?,
            normalized_text: row.get(4)?,
            source_app: row.get(5)?,
            source_window: row.get(6)?,
            hash: row.get(7)?,
            size_bytes: row.get(8)?,
            is_pinned: row.get::<_, i64>(9)? == 1,
            sensitive_score: row.get(10)?,
            copied_at: row.get(11)?,
            created_at: row.get(12)?,
            updated_at: row.get(13)?,
            used_count: row.get(14)?,
            sync_id: row.get(15)?,
            device_id: row.get(16)?,
            revision: row.get(17)?,
            deleted_at: row.get(18)?,
            sync_status: row.get(19)?,
            last_synced_at: row.get(20)?,
            formats: Vec::new(),
            tags: Vec::new(),
        })
    })?;

    let mut items = rows.collect::<Result<Vec<_>, _>>()?;
    hydrate_export_item_associations(connection, &mut items)?;

    Ok(items)
}

fn hydrate_export_item_associations(
    connection: &Connection,
    items: &mut [SyncPackageItem],
) -> Result<(), CliplyError> {
    if items.is_empty() {
        return Ok(());
    }

    let ids = items
        .iter()
        .map(|item| item.id.as_str())
        .collect::<Vec<_>>();
    let mut formats_by_item = load_export_formats_batch(connection, &ids)?;
    let mut tags_by_item = load_export_tags_batch(connection, &ids)?;

    for item in items {
        item.formats = formats_by_item.remove(item.id.as_str()).unwrap_or_default();
        item.tags = tags_by_item.remove(item.id.as_str()).unwrap_or_default();
    }

    Ok(())
}

fn load_export_formats_batch(
    connection: &Connection,
    ids: &[&str],
) -> Result<HashMap<String, Vec<SyncPackageFormat>>, CliplyError> {
    let mut formats_by_item: HashMap<String, Vec<SyncPackageFormat>> = HashMap::new();
    for chunk in ids.chunks(200) {
        let placeholders = vec!["?"; chunk.len()].join(",");
        let sql = format!(
            "SELECT item_id, format_name, mime_type, data_kind, data_text,
                    COALESCE(size_bytes, 0), COALESCE(priority, 0), created_at
             FROM clipboard_formats
             WHERE item_id IN ({placeholders})
               AND (
                 data_kind IN ('text', 'html', 'external_ref')
                 OR data_text IS NOT NULL
               )
             ORDER BY item_id ASC, priority DESC, created_at ASC"
        );
        let mut statement = connection.prepare(&sql)?;
        let rows = statement.query_map(rusqlite::params_from_iter(chunk.iter()), |row| {
            Ok((
                row.get::<_, String>(0)?,
                SyncPackageFormat {
                    format_name: row.get(1)?,
                    mime_type: row.get(2)?,
                    data_kind: row.get(3)?,
                    data_text: row.get(4)?,
                    size_bytes: row.get(5)?,
                    priority: row.get(6)?,
                    created_at: row.get(7)?,
                },
            ))
        })?;

        for row in rows {
            let (item_id, format) = row?;
            let formats = formats_by_item.entry(item_id).or_default();
            if formats.len() < 20 {
                formats.push(format);
            }
        }
    }

    Ok(formats_by_item)
}

fn load_export_tags_batch(
    connection: &Connection,
    ids: &[&str],
) -> Result<HashMap<String, Vec<String>>, CliplyError> {
    let mut tags_by_item: HashMap<String, Vec<String>> = HashMap::new();
    for chunk in ids.chunks(200) {
        let placeholders = vec!["?"; chunk.len()].join(",");
        let sql = format!(
            "SELECT item_id, tag
             FROM clipboard_tags
             WHERE item_id IN ({placeholders})
             ORDER BY item_id ASC, tag ASC"
        );
        let mut statement = connection.prepare(&sql)?;
        let rows = statement.query_map(rusqlite::params_from_iter(chunk.iter()), |row| {
            Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
        })?;

        for row in rows {
            let (item_id, tag) = row?;
            tags_by_item.entry(item_id).or_default().push(tag);
        }
    }

    Ok(tags_by_item)
}

fn load_export_sync_blobs(
    connection: &Connection,
    exported_at: &str,
) -> Result<Vec<SyncPackageBlob>, CliplyError> {
    let cutoff = export_cutoff_timestamp()?;
    let mut statement = match connection.prepare(
        "SELECT id, item_id, blob_type, remote_path, COALESCE(size_bytes, 0),
                hash, COALESCE(encrypted, 0), sync_status, created_at,
                uploaded_at, deleted_at
         FROM sync_blobs
         WHERE julianday(created_at) <= julianday(?2)
           AND (
             deleted_at IS NULL
             OR COALESCE(sync_status, 'pending') <> 'synced'
             OR (
               sync_status = 'synced'
               AND julianday(deleted_at) >= julianday(?1)
             )
           )
         ORDER BY datetime(created_at) ASC, id ASC
         LIMIT 10000",
    ) {
        Ok(statement) => statement,
        Err(error) if is_missing_sync_blobs_table(&error) => return Ok(Vec::new()),
        Err(error) => return Err(error.into()),
    };
    let rows = statement.query_map(params![cutoff, exported_at], |row| {
        Ok(SyncPackageBlob {
            id: row.get(0)?,
            item_id: row.get(1)?,
            blob_type: row.get(2)?,
            remote_path: row.get(3)?,
            size_bytes: row.get(4)?,
            hash: row.get(5)?,
            encrypted: row.get::<_, i64>(6)? == 1,
            sync_status: row.get(7)?,
            created_at: row.get(8)?,
            uploaded_at: row.get(9)?,
            deleted_at: row.get(10)?,
        })
    })?;

    let mut blobs = Vec::new();
    for row in rows {
        blobs.push(row?);
    }
    Ok(blobs)
}

fn load_export_events(
    connection: &Connection,
    exported_at: &str,
) -> Result<Vec<SyncPackageEvent>, CliplyError> {
    let cutoff = export_cutoff_timestamp()?;
    // Abandoned events are represented by the next full item baseline and never
    // ship on their own. Active pending events always ship; synced history only
    // remains inside the export window.
    let mut statement = connection.prepare(
        "SELECT id, item_id, event_type, payload_json, created_at, synced_at
         FROM sync_events
         WHERE abandoned_at IS NULL
           AND julianday(created_at) <= julianday(?2)
           AND (
             synced_at IS NULL
             OR julianday(created_at) >= julianday(?1)
           )
         ORDER BY datetime(created_at) ASC, id ASC
         LIMIT 10000",
    )?;
    let rows = statement.query_map(params![cutoff, exported_at], |row| {
        Ok(SyncPackageEvent {
            id: row.get(0)?,
            item_id: row.get(1)?,
            event_type: row.get(2)?,
            payload_json: row.get(3)?,
            created_at: row.get(4)?,
            synced_at: row.get(5)?,
        })
    })?;

    let mut events = Vec::new();
    for row in rows {
        events.push(row?);
    }
    Ok(events)
}

fn is_missing_sync_blobs_table(error: &rusqlite::Error) -> bool {
    matches!(
        error,
        rusqlite::Error::SqliteFailure(_, Some(message))
            if message.contains("no such table: sync_blobs")
    )
}

fn normalize_package_path(path: &Path) -> PathBuf {
    if path.extension().and_then(|value| value.to_str()) == Some(SYNC_PACKAGE_EXTENSION) {
        return path.to_path_buf();
    }

    let mut output = path.to_path_buf();
    output.set_extension(SYNC_PACKAGE_EXTENSION);
    output
}

fn get_sync_state_value(connection: &Connection, key: &str) -> Result<Option<String>, CliplyError> {
    let value = connection
        .query_row(
            "SELECT value FROM sync_state WHERE key = ?1",
            params![key],
            |row| row.get(0),
        )
        .optional()?;
    Ok(value)
}

fn set_sync_state_value(
    connection: &Connection,
    key: &str,
    value: &str,
) -> Result<(), CliplyError> {
    let now = current_timestamp()?;
    connection.execute(
        "INSERT INTO sync_state (key, value, updated_at)
         VALUES (?1, ?2, ?3)
         ON CONFLICT(key) DO UPDATE SET
           value = excluded.value,
           updated_at = excluded.updated_at",
        params![key, value, now],
    )?;
    Ok(())
}

fn current_timestamp() -> Result<String, CliplyError> {
    OffsetDateTime::now_utc()
        .format(&time::format_description::well_known::Rfc3339)
        .map_err(|error| CliplyError::StorageUnavailable(error.to_string()))
}

#[cfg(test)]
mod tests {
    use super::{load_export_events, load_export_items, load_export_sync_blobs};
    use rusqlite::{params, Connection};

    #[test]
    fn old_unsynced_tombstones_export_while_old_synced_tombstones_expire() {
        let connection = setup_connection();
        insert_item(
            &connection,
            "pending-old",
            "pending",
            Some("2025-01-01T00:00:00Z"),
            "2025-01-01T00:00:00Z",
        );
        insert_item(
            &connection,
            "abandoned-old",
            "abandoned",
            Some("2025-01-01T00:00:00Z"),
            "2025-01-01T00:00:00Z",
        );
        insert_item(
            &connection,
            "synced-old",
            "synced",
            Some("2025-01-01T00:00:00Z"),
            "2025-01-01T00:00:00Z",
        );
        insert_blob(
            &connection,
            "blob-pending-old",
            "pending-old",
            "pending",
            "2025-01-01T00:00:00Z",
            Some("2025-01-01T00:00:00Z"),
        );
        insert_blob(
            &connection,
            "blob-synced-old",
            "synced-old",
            "synced",
            "2025-01-01T00:00:00Z",
            Some("2025-01-01T00:00:00Z"),
        );

        let items =
            load_export_items(&connection, "2026-08-14T08:00:00Z").expect("items should load");
        let item_ids = items.into_iter().map(|item| item.id).collect::<Vec<_>>();
        assert!(item_ids.contains(&"pending-old".to_string()));
        assert!(item_ids.contains(&"abandoned-old".to_string()));
        assert!(!item_ids.contains(&"synced-old".to_string()));

        let blobs =
            load_export_sync_blobs(&connection, "2026-08-14T08:00:00Z").expect("blobs should load");
        let blob_ids = blobs.into_iter().map(|blob| blob.id).collect::<Vec<_>>();
        assert!(blob_ids.contains(&"blob-pending-old".to_string()));
        assert!(!blob_ids.contains(&"blob-synced-old".to_string()));
    }

    #[test]
    fn export_items_batch_associations_and_keep_top_twenty_formats() {
        let connection = setup_connection();
        insert_item(
            &connection,
            "item-formats",
            "pending",
            None,
            "2025-01-01T00:00:00Z",
        );
        insert_item(
            &connection,
            "item-empty",
            "pending",
            None,
            "2025-01-01T00:00:01Z",
        );

        for index in 0..25 {
            connection
                .execute(
                    "INSERT INTO clipboard_formats (
                        id, item_id, format_name, data_kind, data_text,
                        size_bytes, priority, created_at
                     ) VALUES (?1, 'item-formats', ?2, 'text', ?2, ?3, ?3, ?4)",
                    params![
                        format!("format-id-{index}"),
                        format!("format-{index}"),
                        index,
                        format!("2025-01-01T00:00:{index:02}Z")
                    ],
                )
                .expect("format should insert");
        }
        connection
            .execute_batch(
                "INSERT INTO clipboard_tags (item_id, tag) VALUES
                   ('item-formats', 'zeta'),
                   ('item-formats', 'alpha');",
            )
            .expect("tags should insert");

        let items =
            load_export_items(&connection, "2026-08-14T08:00:00Z").expect("items should load");
        let item = items
            .iter()
            .find(|item| item.id == "item-formats")
            .expect("formatted item should export");
        assert_eq!(item.formats.len(), 20);
        assert_eq!(
            item.formats
                .first()
                .map(|format| format.format_name.as_str()),
            Some("format-24")
        );
        assert_eq!(
            item.formats
                .last()
                .map(|format| format.format_name.as_str()),
            Some("format-5")
        );
        assert_eq!(item.tags, vec!["alpha".to_string(), "zeta".to_string()]);

        let empty_item = items
            .iter()
            .find(|item| item.id == "item-empty")
            .expect("empty item should export");
        assert!(empty_item.formats.is_empty());
        assert!(empty_item.tags.is_empty());
    }

    #[test]
    fn abandoned_events_and_changes_after_export_start_are_not_exported() {
        let connection = setup_connection();
        connection
            .execute_batch(
                "INSERT INTO sync_events (
                    id, item_id, event_type, created_at, synced_at, abandoned_at
                 ) VALUES
                   ('active-old', NULL, 'item_updated', '2025-01-01T00:00:00Z', NULL, NULL),
                   ('abandoned-old', NULL, 'item_updated', '2025-01-01T00:00:00Z', NULL, '2026-08-01T00:00:00Z'),
                   ('active-new', NULL, 'item_updated', '2026-08-14T08:00:00.900Z', NULL, NULL);",
            )
            .expect("events should insert");
        insert_item(
            &connection,
            "item-new",
            "pending",
            None,
            "2026-08-14T08:00:00.900Z",
        );
        insert_blob(
            &connection,
            "blob-new",
            "item-new",
            "pending",
            "2026-08-14T08:00:00.900Z",
            None,
        );

        let events = load_export_events(&connection, "2026-08-14T08:00:00.500Z")
            .expect("events should load");
        let event_ids = events.into_iter().map(|event| event.id).collect::<Vec<_>>();
        assert!(event_ids.contains(&"active-old".to_string()));
        assert!(!event_ids.contains(&"abandoned-old".to_string()));
        assert!(!event_ids.contains(&"active-new".to_string()));

        let items =
            load_export_items(&connection, "2026-08-14T08:00:00.500Z").expect("items should load");
        assert!(!items.iter().any(|item| item.id == "item-new"));
        let blobs = load_export_sync_blobs(&connection, "2026-08-14T08:00:00.500Z")
            .expect("blobs should load");
        assert!(!blobs.iter().any(|blob| blob.id == "blob-new"));
    }

    fn setup_connection() -> Connection {
        let connection = Connection::open_in_memory().expect("in-memory sqlite should open");
        connection
            .execute_batch(
                "
                CREATE TABLE clipboard_items (
                  id TEXT PRIMARY KEY,
                  type TEXT NOT NULL,
                  title TEXT,
                  preview_text TEXT,
                  normalized_text TEXT,
                  source_app TEXT,
                  source_window TEXT,
                  hash TEXT NOT NULL,
                  size_bytes INTEGER DEFAULT 0,
                  is_pinned INTEGER DEFAULT 0,
                  sensitive_score INTEGER DEFAULT 0,
                  copied_at TEXT NOT NULL,
                  created_at TEXT NOT NULL,
                  updated_at TEXT NOT NULL,
                  used_count INTEGER DEFAULT 0,
                  sync_id TEXT,
                  device_id TEXT,
                  revision INTEGER DEFAULT 1,
                  deleted_at TEXT NULL,
                  sync_status TEXT DEFAULT 'pending',
                  last_synced_at TEXT NULL
                );
                CREATE TABLE clipboard_formats (
                  id TEXT PRIMARY KEY,
                  item_id TEXT NOT NULL,
                  format_name TEXT NOT NULL,
                  mime_type TEXT,
                  data_kind TEXT NOT NULL,
                  data_text TEXT,
                  size_bytes INTEGER DEFAULT 0,
                  priority INTEGER DEFAULT 0,
                  created_at TEXT NOT NULL
                );
                CREATE TABLE clipboard_tags (
                  item_id TEXT NOT NULL,
                  tag TEXT NOT NULL,
                  PRIMARY KEY (item_id, tag)
                );
                CREATE TABLE sync_blobs (
                  id TEXT PRIMARY KEY,
                  item_id TEXT NOT NULL,
                  blob_type TEXT NOT NULL,
                  remote_path TEXT,
                  size_bytes INTEGER DEFAULT 0,
                  hash TEXT NOT NULL,
                  encrypted INTEGER DEFAULT 0,
                  sync_status TEXT DEFAULT 'pending',
                  created_at TEXT NOT NULL,
                  uploaded_at TEXT NULL,
                  deleted_at TEXT NULL
                );
                CREATE TABLE sync_events (
                  id TEXT PRIMARY KEY,
                  item_id TEXT,
                  event_type TEXT NOT NULL,
                  payload_json TEXT,
                  created_at TEXT NOT NULL,
                  synced_at TEXT NULL,
                  abandoned_at TEXT NULL
                );
                ",
            )
            .expect("export schema should initialize");
        connection
    }

    fn insert_item(
        connection: &Connection,
        id: &str,
        status: &str,
        deleted_at: Option<&str>,
        updated_at: &str,
    ) {
        connection
            .execute(
                "INSERT INTO clipboard_items (
                    id, type, title, hash, copied_at, created_at, updated_at,
                    sync_id, device_id, deleted_at, sync_status
                 ) VALUES (
                    ?1, 'text', ?1, ?1, '2025-01-01T00:00:00Z',
                    '2025-01-01T00:00:00Z', ?4, ?1, 'device-test', ?3, ?2
                 )",
                params![id, status, deleted_at, updated_at],
            )
            .expect("item should insert");
    }

    fn insert_blob(
        connection: &Connection,
        id: &str,
        item_id: &str,
        status: &str,
        created_at: &str,
        deleted_at: Option<&str>,
    ) {
        connection
            .execute(
                "INSERT INTO sync_blobs (
                    id, item_id, blob_type, hash, sync_status, created_at, deleted_at
                 ) VALUES (?1, ?2, 'preview', ?1, ?3, ?4, ?5)",
                params![id, item_id, status, created_at, deleted_at],
            )
            .expect("blob should insert");
    }
}
