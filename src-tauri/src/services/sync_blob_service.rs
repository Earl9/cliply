use crate::error::CliplyError;
use crate::services::blob_service::PreparedSyncImageBlob;
use crate::services::{hash_service, sync_crypto_service};
use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
use tauri::{AppHandle, Manager};
use uuid::Uuid;

pub const REMOTE_BLOB_EXTENSION: &str = "cliply-blob";
pub const REMOTE_BLOB_VERSION: u32 = 1;

pub fn insert_image_sync_blobs(
    connection: &Connection,
    item_id: &str,
    blobs: &[PreparedSyncImageBlob],
    created_at: &str,
) -> Result<usize, CliplyError> {
    let mut inserted = 0;
    for blob in blobs {
        let changed = connection.execute(
            "INSERT OR IGNORE INTO sync_blobs (
                id, item_id, blob_type, local_path, remote_path, size_bytes, hash,
                encrypted, sync_status, created_at, uploaded_at, deleted_at
             ) VALUES (?1, ?2, ?3, ?4, NULL, ?5, ?6, 0, 'pending', ?7, NULL, NULL)",
            params![
                Uuid::new_v4().to_string(),
                item_id,
                blob.blob_type,
                blob.local_path.to_string_lossy().to_string(),
                blob.size_bytes,
                blob.hash,
                created_at
            ],
        )?;
        inserted += changed;
    }

    Ok(inserted)
}

pub fn mark_item_blobs_deleted(
    connection: &Connection,
    item_id: &str,
    deleted_at: &str,
) -> Result<(), CliplyError> {
    match connection.execute(
        "UPDATE sync_blobs
         SET deleted_at = ?2,
             sync_status = 'pending'
         WHERE item_id = ?1
           AND deleted_at IS NULL",
        params![item_id, deleted_at],
    ) {
        Ok(_) => {}
        Err(error) if is_missing_sync_blobs_table(&error) => {}
        Err(error) => return Err(error.into()),
    }
    Ok(())
}

#[derive(Debug, Clone)]
pub struct SyncBlobRecord {
    pub id: String,
    pub item_id: String,
    pub blob_type: String,
    pub local_path: Option<String>,
    pub remote_path: Option<String>,
    pub size_bytes: i64,
    pub hash: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RemoteBlobEnvelope {
    pub version: u32,
    pub app: String,
    pub blob_id: String,
    pub item_id: String,
    pub blob_type: String,
    pub hash: String,
    pub size_bytes: i64,
    pub file_extension: String,
    pub encryption: sync_crypto_service::SyncEncryptionMetadata,
    pub encrypted_payload: String,
}

pub fn load_uploadable_blobs(connection: &Connection) -> Result<Vec<SyncBlobRecord>, CliplyError> {
    let mut statement = match connection.prepare(
        "SELECT id, item_id, blob_type, local_path, remote_path,
                COALESCE(size_bytes, 0), hash
         FROM sync_blobs
         WHERE deleted_at IS NULL
           AND local_path IS NOT NULL
           AND COALESCE(local_path, '') <> ''
           AND (remote_path IS NULL OR sync_status = 'pending')
         ORDER BY datetime(created_at) ASC, id ASC
         LIMIT 200",
    ) {
        Ok(statement) => statement,
        Err(error) if is_missing_sync_blobs_table(&error) => return Ok(Vec::new()),
        Err(error) => return Err(error.into()),
    };

    collect_blob_records(&mut statement)
}

pub fn load_missing_local_blobs(
    connection: &Connection,
) -> Result<Vec<SyncBlobRecord>, CliplyError> {
    let mut statement = match connection.prepare(
        "SELECT id, item_id, blob_type, local_path, remote_path,
                COALESCE(size_bytes, 0), hash
         FROM sync_blobs
         WHERE deleted_at IS NULL
           AND remote_path IS NOT NULL
           AND COALESCE(remote_path, '') <> ''
         ORDER BY datetime(created_at) ASC, id ASC
         LIMIT 200",
    ) {
        Ok(statement) => statement,
        Err(error) if is_missing_sync_blobs_table(&error) => return Ok(Vec::new()),
        Err(error) => return Err(error.into()),
    };

    Ok(collect_blob_records(&mut statement)?
        .into_iter()
        .filter(|record| local_blob_file_missing(record.local_path.as_deref()))
        .collect())
}

pub fn pending_blob_change_count(connection: &Connection) -> Result<i64, CliplyError> {
    match connection.query_row(
        "SELECT COUNT(*)
         FROM sync_blobs
         WHERE sync_status = 'pending'",
        [],
        |row| row.get(0),
    ) {
        Ok(count) => Ok(count),
        Err(error) if is_missing_sync_blobs_table(&error) => Ok(0),
        Err(error) => Err(error.into()),
    }
}

pub fn mark_blob_uploaded(
    connection: &Connection,
    id: &str,
    remote_path: &str,
    uploaded_at: &str,
) -> Result<(), CliplyError> {
    connection.execute(
        "UPDATE sync_blobs
         SET remote_path = ?2,
             encrypted = 1,
             sync_status = 'synced',
             uploaded_at = ?3
         WHERE id = ?1",
        params![id, remote_path, uploaded_at],
    )?;
    Ok(())
}

pub fn mark_pending_blob_tombstones_exported(
    connection: &Connection,
    exported_at: &str,
) -> Result<(), CliplyError> {
    match connection.execute(
        "UPDATE sync_blobs
         SET sync_status = 'synced',
             uploaded_at = COALESCE(uploaded_at, ?1)
         WHERE sync_status = 'pending'
           AND deleted_at IS NOT NULL",
        params![exported_at],
    ) {
        Ok(_) => Ok(()),
        Err(error) if is_missing_sync_blobs_table(&error) => Ok(()),
        Err(error) => Err(error.into()),
    }
}

pub fn write_downloaded_blob(
    app: &AppHandle,
    connection: &Connection,
    record: &SyncBlobRecord,
    envelope: &RemoteBlobEnvelope,
    plaintext: &[u8],
) -> Result<(), CliplyError> {
    if hash_service::stable_bytes_hash(plaintext) != envelope.hash {
        return Err(CliplyError::Sync("图片同步 blob 校验失败".to_string()));
    }

    let local_path = local_sync_blob_path(app, record, &envelope.file_extension)?;
    if let Some(parent) = local_path.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(&local_path, plaintext)?;
    connection.execute(
        "UPDATE sync_blobs
         SET local_path = ?2,
             size_bytes = ?3,
             hash = ?4,
             encrypted = 1,
             sync_status = 'synced'
         WHERE id = ?1",
        params![
            record.id,
            local_path.to_string_lossy().to_string(),
            plaintext.len() as i64,
            envelope.hash
        ],
    )?;
    refresh_item_display_size_from_available_image(connection, &record.item_id)?;
    Ok(())
}

pub fn refresh_item_display_size_from_available_image(
    connection: &Connection,
    item_id: &str,
) -> Result<(), CliplyError> {
    if !clipboard_item_is_image(connection, item_id)? {
        return Ok(());
    }

    let Some(size_bytes) = preferred_local_image_size(connection, item_id)? else {
        return Ok(());
    };

    connection.execute(
        "UPDATE clipboard_items
         SET size_bytes = ?2
         WHERE id = ?1
           AND type = 'image'",
        params![item_id, size_bytes],
    )?;
    Ok(())
}

pub fn available_image_display_size(
    connection: &Connection,
    item_id: &str,
) -> Result<Option<i64>, CliplyError> {
    preferred_local_image_size(connection, item_id)
}

pub fn build_remote_blob_envelope(
    record: &SyncBlobRecord,
    password: &str,
) -> Result<Vec<u8>, CliplyError> {
    let local_path = record
        .local_path
        .as_deref()
        .ok_or_else(|| CliplyError::Sync("图片同步 blob 缺少本地文件".to_string()))?;
    let payload = fs::read(local_path)?;
    let hash = hash_service::stable_bytes_hash(&payload);
    if hash != record.hash {
        return Err(CliplyError::Sync(
            "图片同步 blob 本地文件校验失败".to_string(),
        ));
    }
    if record.size_bytes > 0 && payload.len() as i64 != record.size_bytes {
        return Err(CliplyError::Sync(
            "image sync blob local file size does not match metadata".to_string(),
        ));
    }

    let (encryption, encrypted_payload) = sync_crypto_service::encrypt_payload(&payload, password)?;
    let envelope = RemoteBlobEnvelope {
        version: REMOTE_BLOB_VERSION,
        app: "Cliply".to_string(),
        blob_id: record.id.clone(),
        item_id: record.item_id.clone(),
        blob_type: record.blob_type.clone(),
        hash,
        size_bytes: payload.len() as i64,
        file_extension: file_extension(local_path),
        encryption,
        encrypted_payload,
    };
    serde_json::to_vec_pretty(&envelope)
        .map_err(|error| CliplyError::Sync(format!("图片同步 blob 序列化失败: {error}")))
}

pub fn decrypt_remote_blob_envelope(
    bytes: &[u8],
    password: &str,
) -> Result<(RemoteBlobEnvelope, Vec<u8>), CliplyError> {
    let envelope: RemoteBlobEnvelope = serde_json::from_slice(bytes)
        .map_err(|_| CliplyError::Sync("图片同步 blob 格式不正确".to_string()))?;
    if envelope.app != "Cliply" || envelope.version != REMOTE_BLOB_VERSION {
        return Err(CliplyError::Sync("图片同步 blob 版本不兼容".to_string()));
    }
    let plaintext = sync_crypto_service::decrypt_payload(
        &envelope.encryption,
        &envelope.encrypted_payload,
        password,
    )?;
    if plaintext.len() as i64 != envelope.size_bytes
        || hash_service::stable_bytes_hash(&plaintext) != envelope.hash
    {
        return Err(CliplyError::Sync("图片同步 blob 校验失败".to_string()));
    }

    Ok((envelope, plaintext))
}

pub fn remote_blob_path(record: &SyncBlobRecord) -> String {
    format!(
        "CliplySync/blobs/{}-{}.{}",
        sanitize_path_part(&record.hash),
        sanitize_path_part(&record.id),
        REMOTE_BLOB_EXTENSION
    )
}

fn collect_blob_records(
    statement: &mut rusqlite::Statement<'_>,
) -> Result<Vec<SyncBlobRecord>, CliplyError> {
    let rows = statement.query_map([], |row| {
        Ok(SyncBlobRecord {
            id: row.get(0)?,
            item_id: row.get(1)?,
            blob_type: row.get(2)?,
            local_path: row.get(3)?,
            remote_path: row.get(4)?,
            size_bytes: row.get(5)?,
            hash: row.get(6)?,
        })
    })?;

    let mut records = Vec::new();
    for row in rows {
        records.push(row?);
    }
    Ok(records)
}

fn clipboard_item_is_image(connection: &Connection, item_id: &str) -> Result<bool, CliplyError> {
    let result = connection.query_row(
        "SELECT type = 'image'
         FROM clipboard_items
         WHERE id = ?1
         LIMIT 1",
        params![item_id],
        |row| row.get::<_, i64>(0),
    );

    match result {
        Ok(value) => Ok(value == 1),
        Err(rusqlite::Error::QueryReturnedNoRows) => Ok(false),
        Err(error) => Err(error.into()),
    }
}

fn preferred_local_image_size(
    connection: &Connection,
    item_id: &str,
) -> Result<Option<i64>, CliplyError> {
    if let Some(size_bytes) = primary_clipboard_image_file_size(connection, item_id)? {
        return Ok(Some(size_bytes));
    }

    match preferred_sync_blob_image_size(connection, item_id) {
        Ok(size_bytes) => Ok(size_bytes),
        Err(error) if is_missing_sync_blobs_table(&error) => Ok(None),
        Err(error) => Err(error.into()),
    }
}

fn primary_clipboard_image_file_size(
    connection: &Connection,
    item_id: &str,
) -> Result<Option<i64>, CliplyError> {
    let mut statement = connection.prepare(
        "SELECT data_path, COALESCE(size_bytes, 0)
         FROM clipboard_formats
         WHERE item_id = ?1
           AND data_kind = 'image_file'
           AND format_name <> 'thumbnail/png'
           AND data_path IS NOT NULL
         ORDER BY priority DESC, created_at ASC
         LIMIT 5",
    )?;
    let rows = statement.query_map(params![item_id], |row| {
        Ok((row.get::<_, String>(0)?, row.get::<_, i64>(1)?))
    })?;

    for row in rows {
        let (path, stored_size) = row?;
        if Path::new(&path).exists() {
            let actual_size = fs::metadata(&path)
                .map(|metadata| metadata.len() as i64)
                .unwrap_or(stored_size);
            return Ok(Some(actual_size.max(0)));
        }
    }

    Ok(None)
}

fn preferred_sync_blob_image_size(
    connection: &Connection,
    item_id: &str,
) -> rusqlite::Result<Option<i64>> {
    let mut statement = connection.prepare(
        "SELECT local_path, COALESCE(size_bytes, 0)
         FROM sync_blobs
         WHERE item_id = ?1
           AND blob_type IN ('original', 'compressed', 'preview')
           AND local_path IS NOT NULL
           AND COALESCE(local_path, '') <> ''
           AND deleted_at IS NULL
         ORDER BY CASE blob_type
           WHEN 'original' THEN 0
           WHEN 'compressed' THEN 1
           ELSE 2
         END,
         created_at DESC
         LIMIT 10",
    )?;
    let rows = statement.query_map(params![item_id], |row| {
        Ok((row.get::<_, String>(0)?, row.get::<_, i64>(1)?))
    })?;

    for row in rows {
        let (path, stored_size) = row?;
        if Path::new(&path).exists() {
            let actual_size = fs::metadata(&path)
                .map(|metadata| metadata.len() as i64)
                .unwrap_or(stored_size);
            return Ok(Some(actual_size.max(0)));
        }
    }

    Ok(None)
}

fn local_blob_file_missing(path: Option<&str>) -> bool {
    path.map(str::trim)
        .filter(|path| !path.is_empty())
        .map(|path| !Path::new(path).exists())
        .unwrap_or(true)
}

fn local_sync_blob_path(
    app: &AppHandle,
    record: &SyncBlobRecord,
    extension: &str,
) -> Result<PathBuf, CliplyError> {
    let app_data_dir = app
        .path()
        .app_data_dir()
        .map_err(|error| CliplyError::StorageUnavailable(error.to_string()))?;
    let extension = sanitize_extension(extension);
    Ok(app_data_dir.join("blobs").join("sync-images").join(format!(
        "{}-{}.{}",
        sanitize_path_part(&record.id),
        sanitize_path_part(&record.blob_type),
        extension
    )))
}

fn file_extension(path: &str) -> String {
    Path::new(path)
        .extension()
        .and_then(|value| value.to_str())
        .map(sanitize_extension)
        .unwrap_or_else(|| "bin".to_string())
}

fn sanitize_extension(value: &str) -> String {
    let extension = value
        .trim_start_matches('.')
        .chars()
        .filter(|character| character.is_ascii_alphanumeric())
        .take(12)
        .collect::<String>();
    if extension.is_empty() {
        "bin".to_string()
    } else {
        extension.to_ascii_lowercase()
    }
}

fn sanitize_path_part(value: &str) -> String {
    value
        .chars()
        .map(|character| {
            if character.is_ascii_alphanumeric() || matches!(character, '-' | '_') {
                character
            } else {
                '-'
            }
        })
        .collect()
}

fn is_missing_sync_blobs_table(error: &rusqlite::Error) -> bool {
    matches!(
        error,
        rusqlite::Error::SqliteFailure(_, Some(message))
            if message.contains("no such table: sync_blobs")
    )
}

#[cfg(test)]
mod tests {
    use super::{
        available_image_display_size, build_remote_blob_envelope, decrypt_remote_blob_envelope,
        refresh_item_display_size_from_available_image, remote_blob_path, SyncBlobRecord,
        REMOTE_BLOB_EXTENSION,
    };
    use crate::services::hash_service;
    use rusqlite::{params, Connection};
    use std::fs;

    #[test]
    fn remote_blob_envelope_encrypts_payload_and_roundtrips() {
        let root =
            std::env::temp_dir().join(format!("cliply-sync-blob-test-{}", uuid::Uuid::new_v4()));
        fs::create_dir_all(&root).expect("temp dir should create");
        let local_path = root.join("preview.jpg");
        let payload = b"fake image bytes that should not be visible";
        fs::write(&local_path, payload).expect("blob file should write");

        let record = SyncBlobRecord {
            id: "blob-1".to_string(),
            item_id: "item-1".to_string(),
            blob_type: "preview".to_string(),
            local_path: Some(local_path.to_string_lossy().to_string()),
            remote_path: None,
            size_bytes: payload.len() as i64,
            hash: hash_service::stable_bytes_hash(payload),
        };

        let envelope =
            build_remote_blob_envelope(&record, "sync-password").expect("blob should encrypt");
        let envelope_text = String::from_utf8(envelope.clone()).expect("envelope should be json");
        assert!(!envelope_text.contains("fake image bytes"));

        let (metadata, decrypted) =
            decrypt_remote_blob_envelope(&envelope, "sync-password").expect("blob should decrypt");
        assert_eq!(metadata.blob_id, "blob-1");
        assert_eq!(decrypted, payload);

        fs::remove_dir_all(root).ok();
    }

    #[test]
    fn remote_blob_path_is_scoped_under_cliply_blobs() {
        let record = SyncBlobRecord {
            id: "blob:id".to_string(),
            item_id: "item-1".to_string(),
            blob_type: "preview".to_string(),
            local_path: None,
            remote_path: None,
            size_bytes: 0,
            hash: "hash/value".to_string(),
        };

        let path = remote_blob_path(&record);
        assert!(path.starts_with("CliplySync/blobs/"));
        assert!(path.ends_with(REMOTE_BLOB_EXTENSION));
        assert!(!path.contains(':'));
    }

    #[test]
    fn image_display_size_uses_downloaded_compressed_blob_when_original_is_missing() {
        let connection = setup_image_blob_connection();
        let root =
            std::env::temp_dir().join(format!("cliply-sync-size-test-{}", uuid::Uuid::new_v4()));
        fs::create_dir_all(&root).expect("temp dir should create");
        let compressed_path = root.join("compressed.jpg");
        fs::write(&compressed_path, vec![42_u8; 128_000]).expect("compressed blob should write");

        insert_image_item(&connection, "image-1", 6_100_000);
        connection
            .execute(
                "INSERT INTO clipboard_formats (
                    id, item_id, format_name, data_kind, data_path, size_bytes, priority, created_at
                 ) VALUES (
                    'image-1-original', 'image-1', 'image/bmp', 'image_file',
                    ?1, 6100000, 100, '2026-05-07T00:00:00Z'
                 )",
                params![root.join("missing.bmp").to_string_lossy().to_string()],
            )
            .expect("missing original format should insert");
        connection
            .execute(
                "INSERT INTO sync_blobs (
                    id, item_id, blob_type, local_path, remote_path, size_bytes, hash,
                    encrypted, sync_status, created_at
                 ) VALUES (
                    'blob-1', 'image-1', 'compressed', ?1, 'CliplySync/blobs/blob-1.cliply-blob',
                    128000, 'hash', 1, 'synced', '2026-05-07T00:00:00Z'
                 )",
                params![compressed_path.to_string_lossy().to_string()],
            )
            .expect("sync blob should insert");

        refresh_item_display_size_from_available_image(&connection, "image-1")
            .expect("display size should refresh");

        assert_eq!(item_size(&connection, "image-1"), 128_000);
        fs::remove_dir_all(root).ok();
    }

    #[test]
    fn image_display_size_keeps_original_size_when_original_file_exists() {
        let connection = setup_image_blob_connection();
        let root =
            std::env::temp_dir().join(format!("cliply-sync-size-test-{}", uuid::Uuid::new_v4()));
        fs::create_dir_all(&root).expect("temp dir should create");
        let original_path = root.join("original.bmp");
        let compressed_path = root.join("compressed.jpg");
        fs::write(&original_path, vec![12_u8; 256_000]).expect("original should write");
        fs::write(&compressed_path, vec![42_u8; 64_000]).expect("compressed should write");

        insert_image_item(&connection, "image-1", 6_100_000);
        connection
            .execute(
                "INSERT INTO clipboard_formats (
                    id, item_id, format_name, data_kind, data_path, size_bytes, priority, created_at
                 ) VALUES (
                    'image-1-original', 'image-1', 'image/bmp', 'image_file',
                    ?1, 6100000, 100, '2026-05-07T00:00:00Z'
                 )",
                params![original_path.to_string_lossy().to_string()],
            )
            .expect("original format should insert");
        connection
            .execute(
                "INSERT INTO sync_blobs (
                    id, item_id, blob_type, local_path, remote_path, size_bytes, hash,
                    encrypted, sync_status, created_at
                 ) VALUES (
                    'blob-1', 'image-1', 'compressed', ?1, 'CliplySync/blobs/blob-1.cliply-blob',
                    64000, 'hash', 1, 'synced', '2026-05-07T00:00:00Z'
                 )",
                params![compressed_path.to_string_lossy().to_string()],
            )
            .expect("sync blob should insert");

        let display_size =
            available_image_display_size(&connection, "image-1").expect("display size should load");

        assert_eq!(display_size, Some(256_000));
        fs::remove_dir_all(root).ok();
    }

    fn setup_image_blob_connection() -> Connection {
        let connection = Connection::open_in_memory().expect("in-memory sqlite should open");
        connection
            .execute_batch(
                "
                CREATE TABLE clipboard_items (
                  id TEXT PRIMARY KEY,
                  type TEXT NOT NULL,
                  size_bytes INTEGER DEFAULT 0
                );

                CREATE TABLE clipboard_formats (
                  id TEXT PRIMARY KEY,
                  item_id TEXT NOT NULL,
                  format_name TEXT NOT NULL,
                  data_kind TEXT NOT NULL,
                  data_path TEXT,
                  size_bytes INTEGER DEFAULT 0,
                  priority INTEGER DEFAULT 0,
                  created_at TEXT NOT NULL
                );

                CREATE TABLE sync_blobs (
                  id TEXT PRIMARY KEY,
                  item_id TEXT NOT NULL,
                  blob_type TEXT NOT NULL,
                  local_path TEXT,
                  remote_path TEXT,
                  size_bytes INTEGER DEFAULT 0,
                  hash TEXT NOT NULL,
                  encrypted INTEGER DEFAULT 0,
                  sync_status TEXT DEFAULT 'pending',
                  created_at TEXT NOT NULL,
                  uploaded_at TEXT NULL,
                  deleted_at TEXT NULL
                );
                ",
            )
            .expect("image blob schema should initialize");
        connection
    }

    fn insert_image_item(connection: &Connection, id: &str, size_bytes: i64) {
        connection
            .execute(
                "INSERT INTO clipboard_items (id, type, size_bytes)
                 VALUES (?1, 'image', ?2)",
                params![id, size_bytes],
            )
            .expect("image item should insert");
    }

    fn item_size(connection: &Connection, id: &str) -> i64 {
        connection
            .query_row(
                "SELECT size_bytes FROM clipboard_items WHERE id = ?1",
                params![id],
                |row| row.get(0),
            )
            .expect("item size should load")
    }
}
