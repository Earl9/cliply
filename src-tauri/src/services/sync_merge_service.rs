use crate::error::CliplyError;
use crate::services::sync_package_service::{
    SyncImportResult, SyncPackageEvent, SyncPackageItem, SyncPackagePayload,
};
use rusqlite::{params, Connection, OptionalExtension};
use serde_json::json;
use uuid::Uuid;

#[derive(Debug, Clone)]
struct LocalItemState {
    id: String,
    revision: i64,
    updated_at: String,
    is_pinned: bool,
    deleted_at: Option<String>,
}

pub fn merge_sync_payload(
    connection: &Connection,
    payload: &SyncPackagePayload,
) -> Result<SyncImportResult, CliplyError> {
    upsert_device(connection, payload)?;

    let mut result = SyncImportResult::default();
    for remote_item in &payload.items {
        merge_item(connection, remote_item, &mut result)?;
    }

    for event in &payload.sync_events {
        upsert_sync_event(connection, event)?;
    }

    Ok(result)
}

fn merge_item(
    connection: &Connection,
    remote_item: &SyncPackageItem,
    result: &mut SyncImportResult,
) -> Result<(), CliplyError> {
    let remote_deleted = remote_item.deleted_at.is_some();

    if let Some(local) = find_local_item_by_sync_id(connection, &remote_item.sync_id)? {
        if remote_wins(&local, remote_item) {
            let keep_local_pinned_tombstone_conflict = remote_deleted && local.is_pinned;
            if keep_local_pinned_tombstone_conflict {
                mark_conflict(connection, &local.id, remote_item)?;
                result.conflicted_count += 1;
                result.skipped_count += 1;
                return Ok(());
            }

            update_existing_item(connection, &local.id, remote_item, local.is_pinned)?;
            if remote_deleted {
                result.deleted_count += 1;
            } else {
                result.updated_count += 1;
            }
        } else if local.revision == remote_item.revision
            && local.updated_at == remote_item.updated_at
            && local.deleted_at == remote_item.deleted_at
        {
            result.skipped_count += 1;
        } else {
            result.skipped_count += 1;
        }
        return Ok(());
    }

    if let Some(local) = find_local_item_by_hash(connection, &remote_item.hash)? {
        if local.deleted_at.is_some() && !remote_deleted {
            update_existing_item(connection, &local.id, remote_item, local.is_pinned)?;
            result.updated_count += 1;
            return Ok(());
        }

        merge_duplicate_by_hash(connection, &local, remote_item)?;
        result.skipped_count += 1;
        return Ok(());
    }

    insert_remote_item(connection, remote_item)?;
    if remote_deleted {
        result.deleted_count += 1;
    } else {
        result.imported_count += 1;
    }

    Ok(())
}

fn remote_wins(local: &LocalItemState, remote: &SyncPackageItem) -> bool {
    remote.revision > local.revision
        || (remote.revision == local.revision && remote.updated_at > local.updated_at)
}

fn update_existing_item(
    connection: &Connection,
    local_id: &str,
    remote_item: &SyncPackageItem,
    local_was_pinned: bool,
) -> Result<(), CliplyError> {
    let merged_pinned = local_was_pinned || remote_item.is_pinned;
    let is_deleted = remote_item.deleted_at.is_some();
    connection.execute(
        "UPDATE clipboard_items
         SET type = ?2,
             title = ?3,
             preview_text = ?4,
             normalized_text = ?5,
             source_app = ?6,
             source_window = ?7,
             hash = ?8,
             size_bytes = ?9,
             is_pinned = ?10,
             sensitive_score = ?11,
             copied_at = ?12,
             created_at = ?13,
             updated_at = ?14,
             used_count = MAX(COALESCE(used_count, 0), ?15),
             sync_id = ?16,
             device_id = ?17,
             revision = ?18,
             deleted_at = ?19,
             is_deleted = ?20,
             sync_status = 'pending',
             last_synced_at = NULL
         WHERE id = ?1",
        params![
            local_id,
            remote_item.item_type,
            remote_item.title,
            remote_item.preview_text,
            remote_item.normalized_text,
            remote_item.source_app,
            remote_item.source_window,
            remote_item.hash,
            remote_item.size_bytes,
            if merged_pinned { 1 } else { 0 },
            remote_item.sensitive_score,
            remote_item.copied_at,
            remote_item.created_at,
            remote_item.updated_at,
            remote_item.used_count,
            remote_item.sync_id,
            remote_item.device_id,
            remote_item.revision,
            remote_item.deleted_at,
            if is_deleted { 1 } else { 0 },
        ],
    )?;

    replace_item_children(connection, local_id, remote_item)?;
    Ok(())
}

fn insert_remote_item(
    connection: &Connection,
    remote_item: &SyncPackageItem,
) -> Result<(), CliplyError> {
    let is_deleted = remote_item.deleted_at.is_some();
    let local_id = unique_item_id(connection, &remote_item.id)?;
    connection.execute(
        "INSERT INTO clipboard_items (
            id, type, title, preview_text, normalized_text, source_app, source_window,
            hash, size_bytes, is_pinned, sensitive_score, copied_at, created_at, updated_at,
            used_count, sync_id, device_id, revision, deleted_at, is_deleted,
            sync_status, last_synced_at
         ) VALUES (
            ?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14,
            ?15, ?16, ?17, ?18, ?19, ?20, 'pending', NULL
         )",
        params![
            local_id,
            remote_item.item_type,
            remote_item.title,
            remote_item.preview_text,
            remote_item.normalized_text,
            remote_item.source_app,
            remote_item.source_window,
            remote_item.hash,
            remote_item.size_bytes,
            if remote_item.is_pinned { 1 } else { 0 },
            remote_item.sensitive_score,
            remote_item.copied_at,
            remote_item.created_at,
            remote_item.updated_at,
            remote_item.used_count,
            remote_item.sync_id,
            remote_item.device_id,
            remote_item.revision,
            remote_item.deleted_at,
            if is_deleted { 1 } else { 0 },
        ],
    )?;

    replace_item_children(connection, &local_id, remote_item)?;
    Ok(())
}

fn merge_duplicate_by_hash(
    connection: &Connection,
    local: &LocalItemState,
    remote_item: &SyncPackageItem,
) -> Result<(), CliplyError> {
    let merged_revision = local.revision.max(remote_item.revision);
    let merged_updated_at = if remote_item.updated_at > local.updated_at {
        remote_item.updated_at.as_str()
    } else {
        local.updated_at.as_str()
    };
    let merged_pinned = local.is_pinned || remote_item.is_pinned;
    let merged_deleted_at = merge_deleted_at(local, remote_item);

    connection.execute(
        "UPDATE clipboard_items
         SET is_pinned = ?2,
             revision = ?3,
             updated_at = ?4,
             deleted_at = ?5,
             is_deleted = ?6,
             sync_status = 'pending'
         WHERE id = ?1",
        params![
            local.id,
            if merged_pinned { 1 } else { 0 },
            merged_revision,
            merged_updated_at,
            merged_deleted_at,
            if merged_deleted_at.is_some() { 1 } else { 0 },
        ],
    )?;

    for tag in &remote_item.tags {
        connection.execute(
            "INSERT OR IGNORE INTO clipboard_tags (item_id, tag, created_at)
             VALUES (?1, ?2, ?3)",
            params![local.id, tag, remote_item.created_at],
        )?;
    }

    Ok(())
}

fn merge_deleted_at(local: &LocalItemState, remote_item: &SyncPackageItem) -> Option<String> {
    if local.is_pinned && remote_item.deleted_at.is_some() {
        return None;
    }

    match (&local.deleted_at, &remote_item.deleted_at) {
        (Some(local_deleted_at), Some(remote_deleted_at)) => {
            Some(local_deleted_at.max(remote_deleted_at).clone())
        }
        (Some(local_deleted_at), None) => Some(local_deleted_at.clone()),
        (None, Some(remote_deleted_at)) => Some(remote_deleted_at.clone()),
        (None, None) => None,
    }
}

fn replace_item_children(
    connection: &Connection,
    item_id: &str,
    remote_item: &SyncPackageItem,
) -> Result<(), CliplyError> {
    connection.execute(
        "DELETE FROM clipboard_formats
         WHERE item_id = ?1
           AND data_kind IN ('text', 'html', 'external_ref')",
        params![item_id],
    )?;
    connection.execute(
        "DELETE FROM clipboard_tags WHERE item_id = ?1",
        params![item_id],
    )?;
    connection.execute(
        "DELETE FROM clipboard_items_fts WHERE item_id = ?1",
        params![item_id],
    )?;

    for (index, format) in remote_item.formats.iter().enumerate() {
        connection.execute(
            "INSERT INTO clipboard_formats (
                id, item_id, format_name, mime_type, data_kind, data_text,
                size_bytes, priority, created_at
             ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
            params![
                format!("{item_id}-sync-format-{index}"),
                item_id,
                format.format_name,
                format.mime_type,
                format.data_kind,
                format.data_text,
                format.size_bytes,
                format.priority,
                format.created_at,
            ],
        )?;
    }

    for tag in &remote_item.tags {
        connection.execute(
            "INSERT OR IGNORE INTO clipboard_tags (item_id, tag, created_at)
             VALUES (?1, ?2, ?3)",
            params![item_id, tag, remote_item.created_at],
        )?;
    }

    if remote_item.deleted_at.is_none() {
        connection.execute(
            "INSERT INTO clipboard_items_fts (
                item_id, title, preview_text, normalized_text, source_app
             ) VALUES (?1, ?2, ?3, ?4, ?5)",
            params![
                item_id,
                remote_item.title.as_deref().unwrap_or_default(),
                remote_item.preview_text.as_deref().unwrap_or_default(),
                remote_item.normalized_text.as_deref().unwrap_or_default(),
                remote_item.source_app.as_deref().unwrap_or_default(),
            ],
        )?;
    }

    Ok(())
}

fn mark_conflict(
    connection: &Connection,
    local_id: &str,
    remote_item: &SyncPackageItem,
) -> Result<(), CliplyError> {
    connection.execute(
        "UPDATE clipboard_items
         SET sync_status = 'conflicted',
             revision = MAX(COALESCE(revision, 1), ?2)
         WHERE id = ?1",
        params![local_id, remote_item.revision],
    )?;

    connection.execute(
        "INSERT INTO sync_events (
            id, item_id, event_type, payload_json, created_at, synced_at
         ) VALUES (?1, ?2, 'item_conflicted', ?3, ?4, NULL)",
        params![
            Uuid::new_v4().to_string(),
            local_id,
            json!({
                "reason": "remote_tombstone_preserved_local_pinned",
                "remoteSyncId": remote_item.sync_id,
                "remoteRevision": remote_item.revision,
                "remoteDeletedAt": remote_item.deleted_at
            })
            .to_string(),
            remote_item.updated_at,
        ],
    )?;

    Ok(())
}

fn upsert_device(connection: &Connection, payload: &SyncPackagePayload) -> Result<(), CliplyError> {
    connection.execute(
        "INSERT INTO devices (id, name, platform, created_at, last_seen_at)
         VALUES (?1, ?2, ?3, ?4, ?5)
         ON CONFLICT(id) DO UPDATE SET
           name = excluded.name,
           platform = excluded.platform,
           last_seen_at = excluded.last_seen_at",
        params![
            payload.device.id,
            payload.device.name,
            payload.device.platform,
            payload.device.created_at,
            payload.exported_at,
        ],
    )?;
    Ok(())
}

fn upsert_sync_event(connection: &Connection, event: &SyncPackageEvent) -> Result<(), CliplyError> {
    connection.execute(
        "INSERT OR IGNORE INTO sync_events (
            id, item_id, event_type, payload_json, created_at, synced_at
         ) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
        params![
            event.id,
            event.item_id,
            event.event_type,
            event.payload_json,
            event.created_at,
            event.synced_at,
        ],
    )?;
    Ok(())
}

fn find_local_item_by_sync_id(
    connection: &Connection,
    sync_id: &str,
) -> Result<Option<LocalItemState>, CliplyError> {
    find_local_item(
        connection,
        "SELECT id, COALESCE(revision, 1), updated_at, COALESCE(is_pinned, 0), deleted_at
         FROM clipboard_items
         WHERE sync_id = ?1
         LIMIT 1",
        sync_id,
    )
}

fn find_local_item_by_hash(
    connection: &Connection,
    hash: &str,
) -> Result<Option<LocalItemState>, CliplyError> {
    find_local_item(
        connection,
        "SELECT id, COALESCE(revision, 1), updated_at, COALESCE(is_pinned, 0), deleted_at
         FROM clipboard_items
         WHERE hash = ?1
         ORDER BY deleted_at IS NOT NULL ASC, datetime(updated_at) DESC
         LIMIT 1",
        hash,
    )
}

fn find_local_item(
    connection: &Connection,
    sql: &str,
    value: &str,
) -> Result<Option<LocalItemState>, CliplyError> {
    let item = connection
        .query_row(sql, params![value], |row| {
            Ok(LocalItemState {
                id: row.get(0)?,
                revision: row.get(1)?,
                updated_at: row.get(2)?,
                is_pinned: row.get::<_, i64>(3)? == 1,
                deleted_at: row.get(4)?,
            })
        })
        .optional()?;
    Ok(item)
}

fn unique_item_id(connection: &Connection, preferred_id: &str) -> Result<String, CliplyError> {
    let exists: i64 = connection.query_row(
        "SELECT COUNT(*) FROM clipboard_items WHERE id = ?1",
        params![preferred_id],
        |row| row.get(0),
    )?;
    if exists == 0 {
        return Ok(preferred_id.to_string());
    }

    Ok(Uuid::new_v4().to_string())
}

#[cfg(test)]
mod tests {
    use super::merge_sync_payload;
    use crate::services::sync_package_service::{
        SyncPackageDevice, SyncPackageItem, SyncPackagePayload,
    };
    use rusqlite::{params, Connection};

    #[test]
    fn imports_new_items_and_preserves_pin_state() {
        let connection = setup_connection();
        let payload = payload_with_items(vec![item("remote-1", "sync-1", "hash-1", 1, true, None)]);

        let result = merge_sync_payload(&connection, &payload).expect("payload should merge");

        assert_eq!(result.imported_count, 1);
        assert_eq!(pinned(&connection, "remote-1"), 1);
        assert_eq!(visible_count(&connection), 1);
    }

    #[test]
    fn higher_revision_updates_existing_item() {
        let connection = setup_connection();
        insert_local_item(&connection, "local-1", "sync-1", "hash-1", 1, false, None);
        let mut remote = item("remote-1", "sync-1", "hash-1", 2, true, None);
        remote.title = Some("Remote wins".to_string());
        let payload = payload_with_items(vec![remote]);

        let result = merge_sync_payload(&connection, &payload).expect("payload should merge");

        assert_eq!(result.updated_count, 1);
        assert_eq!(title(&connection, "local-1"), "Remote wins");
        assert_eq!(pinned(&connection, "local-1"), 1);
    }

    #[test]
    fn duplicate_hash_is_skipped_and_pin_is_or_merged() {
        let connection = setup_connection();
        insert_local_item(
            &connection,
            "local-1",
            "sync-local",
            "same-hash",
            1,
            false,
            None,
        );
        let payload = payload_with_items(vec![item(
            "remote-1",
            "sync-remote",
            "same-hash",
            1,
            true,
            None,
        )]);

        let result = merge_sync_payload(&connection, &payload).expect("payload should merge");

        assert_eq!(result.skipped_count, 1);
        assert_eq!(item_count(&connection), 1);
        assert_eq!(pinned(&connection, "local-1"), 1);
    }

    #[test]
    fn remote_tombstone_hides_unpinned_item() {
        let connection = setup_connection();
        insert_local_item(&connection, "local-1", "sync-1", "hash-1", 1, false, None);
        let payload = payload_with_items(vec![item(
            "remote-1",
            "sync-1",
            "hash-1",
            2,
            false,
            Some("2026-05-06T12:00:00Z"),
        )]);

        let result = merge_sync_payload(&connection, &payload).expect("payload should merge");

        assert_eq!(result.deleted_count, 1);
        assert_eq!(visible_count(&connection), 0);
    }

    #[test]
    fn remote_tombstone_does_not_delete_local_pinned_item() {
        let connection = setup_connection();
        insert_local_item(&connection, "local-1", "sync-1", "hash-1", 1, true, None);
        let payload = payload_with_items(vec![item(
            "remote-1",
            "sync-1",
            "hash-1",
            2,
            false,
            Some("2026-05-06T12:00:00Z"),
        )]);

        let result = merge_sync_payload(&connection, &payload).expect("payload should merge");

        assert_eq!(result.conflicted_count, 1);
        assert_eq!(visible_count(&connection), 1);
        assert_eq!(sync_status(&connection, "local-1"), "conflicted");
    }

    fn setup_connection() -> Connection {
        let connection = Connection::open_in_memory().expect("sqlite should open");
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
                  is_favorite INTEGER DEFAULT 0,
                  is_deleted INTEGER DEFAULT 0,
                  sensitive_score INTEGER DEFAULT 0,
                  copied_at TEXT NOT NULL,
                  created_at TEXT NOT NULL,
                  updated_at TEXT NOT NULL,
                  used_count INTEGER DEFAULT 0,
                  sync_id TEXT UNIQUE,
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
                  data_path TEXT,
                  size_bytes INTEGER DEFAULT 0,
                  priority INTEGER DEFAULT 0,
                  created_at TEXT NOT NULL
                );
                CREATE TABLE clipboard_tags (
                  item_id TEXT NOT NULL,
                  tag TEXT NOT NULL,
                  created_at TEXT NOT NULL,
                  PRIMARY KEY (item_id, tag)
                );
                CREATE VIRTUAL TABLE clipboard_items_fts USING fts5(
                  item_id UNINDEXED,
                  title,
                  preview_text,
                  normalized_text,
                  source_app
                );
                CREATE TABLE devices (
                  id TEXT PRIMARY KEY,
                  name TEXT NOT NULL,
                  platform TEXT NOT NULL,
                  created_at TEXT NOT NULL,
                  last_seen_at TEXT NULL
                );
                CREATE TABLE sync_state (
                  key TEXT PRIMARY KEY,
                  value TEXT NOT NULL,
                  updated_at TEXT NOT NULL
                );
                CREATE TABLE sync_events (
                  id TEXT PRIMARY KEY,
                  item_id TEXT,
                  event_type TEXT NOT NULL,
                  payload_json TEXT,
                  created_at TEXT NOT NULL,
                  synced_at TEXT NULL
                );
                ",
            )
            .expect("schema should initialize");
        connection
    }

    fn payload_with_items(items: Vec<SyncPackageItem>) -> SyncPackagePayload {
        SyncPackagePayload {
            version: 1,
            app: "Cliply".to_string(),
            exported_at: "2026-05-06T12:00:00Z".to_string(),
            device: SyncPackageDevice {
                id: "device-a".to_string(),
                name: "Device A".to_string(),
                platform: "windows".to_string(),
                created_at: "2026-05-06T00:00:00Z".to_string(),
                last_seen_at: Some("2026-05-06T12:00:00Z".to_string()),
            },
            items,
            sync_events: Vec::new(),
        }
    }

    fn item(
        id: &str,
        sync_id: &str,
        hash: &str,
        revision: i64,
        is_pinned: bool,
        deleted_at: Option<&str>,
    ) -> SyncPackageItem {
        SyncPackageItem {
            id: id.to_string(),
            item_type: "text".to_string(),
            title: Some(id.to_string()),
            preview_text: Some(id.to_string()),
            normalized_text: Some(id.to_string()),
            source_app: Some("Test".to_string()),
            source_window: None,
            hash: hash.to_string(),
            size_bytes: id.len() as i64,
            is_pinned,
            sensitive_score: 0,
            copied_at: "2026-05-06T10:00:00Z".to_string(),
            created_at: "2026-05-06T10:00:00Z".to_string(),
            updated_at: format!("2026-05-06T10:00:0{revision}Z"),
            used_count: 0,
            sync_id: sync_id.to_string(),
            device_id: "device-a".to_string(),
            revision,
            deleted_at: deleted_at.map(ToString::to_string),
            sync_status: Some("pending".to_string()),
            last_synced_at: None,
            formats: Vec::new(),
            tags: Vec::new(),
        }
    }

    fn insert_local_item(
        connection: &Connection,
        id: &str,
        sync_id: &str,
        hash: &str,
        revision: i64,
        is_pinned: bool,
        deleted_at: Option<&str>,
    ) {
        connection
            .execute(
                "INSERT INTO clipboard_items (
                    id, type, title, preview_text, normalized_text, source_app,
                    hash, is_pinned, copied_at, created_at, updated_at, sync_id,
                    device_id, revision, deleted_at, is_deleted
                 ) VALUES (
                    ?1, 'text', ?1, ?1, ?1, 'Test', ?3, ?5,
                    '2026-05-06T09:00:00Z', '2026-05-06T09:00:00Z',
                    '2026-05-06T09:00:00Z', ?2, 'device-local', ?4, ?6, ?7
                 )",
                params![
                    id,
                    sync_id,
                    hash,
                    revision,
                    if is_pinned { 1 } else { 0 },
                    deleted_at,
                    if deleted_at.is_some() { 1 } else { 0 },
                ],
            )
            .expect("item should insert");
    }

    fn item_count(connection: &Connection) -> i64 {
        connection
            .query_row("SELECT COUNT(*) FROM clipboard_items", [], |row| row.get(0))
            .expect("item count should load")
    }

    fn visible_count(connection: &Connection) -> i64 {
        connection
            .query_row(
                "SELECT COUNT(*) FROM clipboard_items WHERE deleted_at IS NULL AND is_deleted = 0",
                [],
                |row| row.get(0),
            )
            .expect("visible count should load")
    }

    fn pinned(connection: &Connection, id: &str) -> i64 {
        connection
            .query_row(
                "SELECT is_pinned FROM clipboard_items WHERE id = ?1",
                params![id],
                |row| row.get(0),
            )
            .expect("pin state should load")
    }

    fn title(connection: &Connection, id: &str) -> String {
        connection
            .query_row(
                "SELECT title FROM clipboard_items WHERE id = ?1",
                params![id],
                |row| row.get(0),
            )
            .expect("title should load")
    }

    fn sync_status(connection: &Connection, id: &str) -> String {
        connection
            .query_row(
                "SELECT sync_status FROM clipboard_items WHERE id = ?1",
                params![id],
                |row| row.get(0),
            )
            .expect("sync status should load")
    }
}
