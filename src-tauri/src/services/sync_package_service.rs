use crate::error::CliplyError;
use crate::services::{database_service, sync_crypto_service, sync_merge_service, sync_service};
use rusqlite::{params, Connection, OptionalExtension};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
use tauri::AppHandle;
use time::OffsetDateTime;

pub const SYNC_PACKAGE_APP: &str = "Cliply";
pub const SYNC_PACKAGE_VERSION: u32 = 1;
pub const SYNC_PACKAGE_EXTENSION: &str = "cliply-sync";

const LAST_EXPORTED_AT_KEY: &str = "last_sync_package_exported_at";
const LAST_IMPORTED_AT_KEY: &str = "last_sync_package_imported_at";

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
    let (package_json, exported_at) = build_sync_package_bytes(app, &password)?;
    let connection = database_service::connect(app)?;
    let output_path = normalize_package_path(Path::new(&path));
    if let Some(parent) = output_path.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(output_path, package_json)?;
    set_sync_state_value(&connection, LAST_EXPORTED_AT_KEY, &exported_at)?;
    Ok(())
}

pub fn build_sync_package_bytes(
    app: &AppHandle,
    password: &str,
) -> Result<(Vec<u8>, String), CliplyError> {
    let connection = database_service::connect(app)?;
    let exported_at = current_timestamp()?;
    let device = load_current_device(&connection)?;
    let payload = SyncPackagePayload {
        version: SYNC_PACKAGE_VERSION,
        app: SYNC_PACKAGE_APP.to_string(),
        exported_at: exported_at.clone(),
        device: device.clone(),
        items: load_export_items(&connection)?,
        sync_blobs: load_export_sync_blobs(&connection)?,
        sync_events: load_export_events(&connection)?,
    };
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
    Ok((package_json, exported_at))
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

fn load_export_items(connection: &Connection) -> Result<Vec<SyncPackageItem>, CliplyError> {
    let mut statement = connection.prepare(
        "SELECT id, type, title, preview_text, normalized_text, source_app, source_window,
                hash, COALESCE(size_bytes, 0), COALESCE(is_pinned, 0),
                COALESCE(sensitive_score, 0), copied_at, created_at, updated_at,
                COALESCE(used_count, 0), COALESCE(sync_id, id), COALESCE(device_id, ''),
                COALESCE(revision, 1), deleted_at, sync_status, last_synced_at
         FROM clipboard_items
         WHERE sync_id IS NOT NULL
         ORDER BY datetime(updated_at) ASC, id ASC",
    )?;
    let rows = statement.query_map([], |row| {
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

    let mut items = Vec::new();
    for row in rows {
        let mut item = row?;
        item.formats = load_export_formats(connection, &item.id)?;
        item.tags = load_export_tags(connection, &item.id)?;
        items.push(item);
    }

    Ok(items)
}

fn load_export_formats(
    connection: &Connection,
    item_id: &str,
) -> Result<Vec<SyncPackageFormat>, CliplyError> {
    let mut statement = connection.prepare(
        "SELECT format_name, mime_type, data_kind, data_text,
                COALESCE(size_bytes, 0), COALESCE(priority, 0), created_at
         FROM clipboard_formats
         WHERE item_id = ?1
           AND (
             data_kind IN ('text', 'html', 'external_ref')
             OR data_text IS NOT NULL
           )
         ORDER BY priority DESC, created_at ASC
         LIMIT 20",
    )?;
    let rows = statement.query_map(params![item_id], |row| {
        Ok(SyncPackageFormat {
            format_name: row.get(0)?,
            mime_type: row.get(1)?,
            data_kind: row.get(2)?,
            data_text: row.get(3)?,
            size_bytes: row.get(4)?,
            priority: row.get(5)?,
            created_at: row.get(6)?,
        })
    })?;

    let mut formats = Vec::new();
    for row in rows {
        formats.push(row?);
    }
    Ok(formats)
}

fn load_export_tags(connection: &Connection, item_id: &str) -> Result<Vec<String>, CliplyError> {
    let mut statement =
        connection.prepare("SELECT tag FROM clipboard_tags WHERE item_id = ?1 ORDER BY tag ASC")?;
    let rows = statement.query_map(params![item_id], |row| row.get(0))?;
    let mut tags = Vec::new();
    for row in rows {
        tags.push(row?);
    }
    Ok(tags)
}

fn load_export_sync_blobs(connection: &Connection) -> Result<Vec<SyncPackageBlob>, CliplyError> {
    let mut statement = match connection.prepare(
        "SELECT id, item_id, blob_type, remote_path, COALESCE(size_bytes, 0),
                hash, COALESCE(encrypted, 0), sync_status, created_at,
                uploaded_at, deleted_at
         FROM sync_blobs
         ORDER BY datetime(created_at) ASC, id ASC
         LIMIT 10000",
    ) {
        Ok(statement) => statement,
        Err(error) if is_missing_sync_blobs_table(&error) => return Ok(Vec::new()),
        Err(error) => return Err(error.into()),
    };
    let rows = statement.query_map([], |row| {
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

fn load_export_events(connection: &Connection) -> Result<Vec<SyncPackageEvent>, CliplyError> {
    let mut statement = connection.prepare(
        "SELECT id, item_id, event_type, payload_json, created_at, synced_at
         FROM sync_events
         ORDER BY datetime(created_at) ASC, id ASC
         LIMIT 10000",
    )?;
    let rows = statement.query_map([], |row| {
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
