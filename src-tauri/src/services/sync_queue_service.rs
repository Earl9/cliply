use crate::error::CliplyError;
use crate::services::database_service;
use rusqlite::{params, Connection, OptionalExtension, TransactionBehavior};
use serde::{Deserialize, Serialize};
use tauri::AppHandle;
use time::OffsetDateTime;

pub const AUTO_SYNC_ENABLED_KEY: &str = "remote_sync_auto_enabled";
const FULL_EXPORT_REQUIRED_KEY: &str = "remote_sync_full_export_required";
const LAST_QUEUE_ABANDONED_AT_KEY: &str = "remote_sync_last_queue_abandoned_at";
const LAST_ABANDONED_ITEM_COUNT_KEY: &str = "remote_sync_last_abandoned_item_count";
const LAST_ABANDONED_TOMBSTONE_COUNT_KEY: &str = "remote_sync_last_abandoned_tombstone_count";
const LAST_ABANDONED_EVENT_COUNT_KEY: &str = "remote_sync_last_abandoned_event_count";
const LAST_ABANDONED_BLOB_COUNT_KEY: &str = "remote_sync_last_abandoned_blob_count";

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct SyncQueueStatus {
    pub pending_item_count: usize,
    pub pending_tombstone_count: usize,
    pub pending_event_count: usize,
    pub pending_blob_count: usize,
    pub full_export_required: bool,
    pub last_queue_abandoned_at: Option<String>,
    pub last_abandoned_item_count: usize,
    pub last_abandoned_tombstone_count: usize,
    pub last_abandoned_event_count: usize,
    pub last_abandoned_blob_count: usize,
}

impl SyncQueueStatus {
    pub fn pending_change_count(&self) -> usize {
        self.pending_item_count
            + self.pending_tombstone_count
            + self.pending_event_count
            + self.pending_blob_count
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SyncExportCheckpoint {
    pub full_export_required: bool,
    pub last_queue_abandoned_at: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SyncQueueAbandonResult {
    pub abandoned_at: String,
    pub item_count: usize,
    pub tombstone_count: usize,
    pub event_count: usize,
    pub blob_count: usize,
}

impl SyncQueueAbandonResult {
    pub fn abandoned_change_count(&self) -> usize {
        self.item_count + self.tombstone_count + self.event_count + self.blob_count
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SyncExportMarkResult {
    pub full_baseline_applied: bool,
    pub remaining_abandoned_count: usize,
}

pub fn load_sync_queue_status(connection: &Connection) -> Result<SyncQueueStatus, CliplyError> {
    let (pending_item_count, pending_tombstone_count) = pending_item_counts(connection)?;
    let pending_event_count = query_count(
        connection,
        "SELECT COUNT(*)
         FROM sync_events
         WHERE synced_at IS NULL
           AND abandoned_at IS NULL",
    )?;
    let pending_blob_count = query_count(
        connection,
        "SELECT COUNT(*)
         FROM sync_blobs
         WHERE COALESCE(sync_status, 'pending') = 'pending'",
    )?;

    Ok(SyncQueueStatus {
        pending_item_count,
        pending_tombstone_count,
        pending_event_count,
        pending_blob_count,
        full_export_required: get_bool_sync_state_value(connection, FULL_EXPORT_REQUIRED_KEY)?,
        last_queue_abandoned_at: get_sync_state_value(connection, LAST_QUEUE_ABANDONED_AT_KEY)?,
        last_abandoned_item_count: get_usize_sync_state_value(
            connection,
            LAST_ABANDONED_ITEM_COUNT_KEY,
        )?,
        last_abandoned_tombstone_count: get_usize_sync_state_value(
            connection,
            LAST_ABANDONED_TOMBSTONE_COUNT_KEY,
        )?,
        last_abandoned_event_count: get_usize_sync_state_value(
            connection,
            LAST_ABANDONED_EVENT_COUNT_KEY,
        )?,
        last_abandoned_blob_count: get_usize_sync_state_value(
            connection,
            LAST_ABANDONED_BLOB_COUNT_KEY,
        )?,
    })
}

pub fn load_sync_export_checkpoint(
    connection: &Connection,
) -> Result<SyncExportCheckpoint, CliplyError> {
    Ok(SyncExportCheckpoint {
        full_export_required: get_bool_sync_state_value(connection, FULL_EXPORT_REQUIRED_KEY)?,
        last_queue_abandoned_at: get_sync_state_value(connection, LAST_QUEUE_ABANDONED_AT_KEY)?,
    })
}

pub fn abandon_sync_queue(app: &AppHandle) -> Result<SyncQueueAbandonResult, CliplyError> {
    let mut connection = database_service::connect(app)?;
    let abandoned_at = current_timestamp()?;
    abandon_sync_queue_at(&mut connection, &abandoned_at)
}

pub fn abandon_sync_queue_at(
    connection: &mut Connection,
    abandoned_at: &str,
) -> Result<SyncQueueAbandonResult, CliplyError> {
    let transaction = connection.transaction_with_behavior(TransactionBehavior::Immediate)?;
    let (item_count, tombstone_count) = pending_item_counts(&transaction)?;
    let event_count = query_count(
        &transaction,
        "SELECT COUNT(*)
         FROM sync_events
         WHERE synced_at IS NULL
           AND abandoned_at IS NULL",
    )?;
    let blob_count = query_count(
        &transaction,
        "SELECT COUNT(*)
         FROM sync_blobs
         WHERE COALESCE(sync_status, 'pending') = 'pending'",
    )?;

    transaction.execute(
        "UPDATE clipboard_items
         SET sync_status = 'abandoned'
         WHERE COALESCE(sync_status, 'pending') = 'pending'",
        [],
    )?;
    transaction.execute(
        "UPDATE sync_blobs
         SET sync_status = 'abandoned'
         WHERE COALESCE(sync_status, 'pending') = 'pending'",
        [],
    )?;
    transaction.execute(
        "UPDATE sync_events
         SET abandoned_at = ?1
         WHERE synced_at IS NULL
           AND abandoned_at IS NULL",
        params![abandoned_at],
    )?;

    set_sync_state_value(&transaction, AUTO_SYNC_ENABLED_KEY, "false", abandoned_at)?;
    set_sync_state_value(&transaction, FULL_EXPORT_REQUIRED_KEY, "true", abandoned_at)?;
    set_sync_state_value(
        &transaction,
        LAST_QUEUE_ABANDONED_AT_KEY,
        abandoned_at,
        abandoned_at,
    )?;
    set_sync_state_value(
        &transaction,
        LAST_ABANDONED_ITEM_COUNT_KEY,
        &item_count.to_string(),
        abandoned_at,
    )?;
    set_sync_state_value(
        &transaction,
        LAST_ABANDONED_TOMBSTONE_COUNT_KEY,
        &tombstone_count.to_string(),
        abandoned_at,
    )?;
    set_sync_state_value(
        &transaction,
        LAST_ABANDONED_EVENT_COUNT_KEY,
        &event_count.to_string(),
        abandoned_at,
    )?;
    set_sync_state_value(
        &transaction,
        LAST_ABANDONED_BLOB_COUNT_KEY,
        &blob_count.to_string(),
        abandoned_at,
    )?;
    transaction.commit()?;

    Ok(SyncQueueAbandonResult {
        abandoned_at: abandoned_at.to_string(),
        item_count,
        tombstone_count,
        event_count,
        blob_count,
    })
}

pub fn mark_sync_exported(
    connection: &mut Connection,
    exported_at: &str,
    checkpoint: &SyncExportCheckpoint,
) -> Result<SyncExportMarkResult, CliplyError> {
    let transaction = connection.transaction_with_behavior(TransactionBehavior::Immediate)?;

    transaction.execute(
        "UPDATE clipboard_items
         SET sync_status = 'synced',
             last_synced_at = ?1
         WHERE COALESCE(sync_status, 'pending') = 'pending'
           AND julianday(updated_at) <= julianday(?1)",
        params![exported_at],
    )?;
    transaction.execute(
        "UPDATE sync_events
         SET synced_at = ?1
         WHERE synced_at IS NULL
           AND abandoned_at IS NULL
           AND julianday(created_at) <= julianday(?1)",
        params![exported_at],
    )?;
    transaction.execute(
        "UPDATE sync_blobs
         SET sync_status = 'synced',
             uploaded_at = COALESCE(uploaded_at, ?1)
         WHERE COALESCE(sync_status, 'pending') = 'pending'
           AND deleted_at IS NOT NULL
           AND julianday(deleted_at) <= julianday(?1)",
        params![exported_at],
    )?;

    let current_checkpoint = load_sync_export_checkpoint(&transaction)?;
    let full_baseline_applied = checkpoint.full_export_required
        && current_checkpoint.full_export_required
        && current_checkpoint.last_queue_abandoned_at == checkpoint.last_queue_abandoned_at;

    if full_baseline_applied {
        transaction.execute(
            "UPDATE clipboard_items
             SET sync_status = 'synced',
                 last_synced_at = ?1
             WHERE sync_status = 'abandoned'
               AND julianday(updated_at) <= julianday(?1)",
            params![exported_at],
        )?;
        transaction.execute(
            "UPDATE sync_events
             SET synced_at = ?1
             WHERE synced_at IS NULL
               AND abandoned_at IS NOT NULL
               AND julianday(abandoned_at) <= julianday(?1)",
            params![exported_at],
        )?;
        transaction.execute(
            "UPDATE sync_blobs
             SET sync_status = 'synced',
                 uploaded_at = COALESCE(uploaded_at, ?1)
             WHERE sync_status = 'abandoned'
               AND (deleted_at IS NOT NULL OR COALESCE(remote_path, '') <> '')
               AND julianday(COALESCE(deleted_at, uploaded_at, created_at)) <= julianday(?1)",
            params![exported_at],
        )?;
    }

    let remaining_abandoned_count = abandoned_change_count(&transaction)?;
    if full_baseline_applied {
        set_sync_state_value(
            &transaction,
            FULL_EXPORT_REQUIRED_KEY,
            if remaining_abandoned_count == 0 {
                "false"
            } else {
                "true"
            },
            exported_at,
        )?;
    }

    transaction.commit()?;
    Ok(SyncExportMarkResult {
        full_baseline_applied,
        remaining_abandoned_count,
    })
}

fn pending_item_counts(connection: &Connection) -> Result<(usize, usize), CliplyError> {
    let (live, tombstones): (i64, i64) = connection.query_row(
        "SELECT
           COALESCE(SUM(CASE WHEN COALESCE(is_deleted, 0) = 0 THEN 1 ELSE 0 END), 0),
           COALESCE(SUM(CASE WHEN COALESCE(is_deleted, 0) = 1 THEN 1 ELSE 0 END), 0)
         FROM clipboard_items
         WHERE COALESCE(sync_status, 'pending') = 'pending'",
        [],
        |row| Ok((row.get(0)?, row.get(1)?)),
    )?;
    Ok((count_to_usize(live), count_to_usize(tombstones)))
}

fn abandoned_change_count(connection: &Connection) -> Result<usize, CliplyError> {
    let item_count = query_count(
        connection,
        "SELECT COUNT(*) FROM clipboard_items WHERE sync_status = 'abandoned'",
    )?;
    let event_count = query_count(
        connection,
        "SELECT COUNT(*)
         FROM sync_events
         WHERE synced_at IS NULL
           AND abandoned_at IS NOT NULL",
    )?;
    let blob_count = query_count(
        connection,
        "SELECT COUNT(*) FROM sync_blobs WHERE sync_status = 'abandoned'",
    )?;
    Ok(item_count + event_count + blob_count)
}

fn query_count(connection: &Connection, sql: &str) -> Result<usize, CliplyError> {
    let count: i64 = connection.query_row(sql, [], |row| row.get(0))?;
    Ok(count_to_usize(count))
}

fn count_to_usize(count: i64) -> usize {
    usize::try_from(count.max(0)).unwrap_or(usize::MAX)
}

fn get_sync_state_value(connection: &Connection, key: &str) -> Result<Option<String>, CliplyError> {
    Ok(connection
        .query_row(
            "SELECT value FROM sync_state WHERE key = ?1",
            params![key],
            |row| row.get(0),
        )
        .optional()?)
}

fn get_bool_sync_state_value(connection: &Connection, key: &str) -> Result<bool, CliplyError> {
    Ok(matches!(
        get_sync_state_value(connection, key)?.as_deref(),
        Some("true") | Some("1")
    ))
}

fn get_usize_sync_state_value(connection: &Connection, key: &str) -> Result<usize, CliplyError> {
    Ok(get_sync_state_value(connection, key)?
        .and_then(|value| value.parse::<usize>().ok())
        .unwrap_or(0))
}

fn set_sync_state_value(
    connection: &Connection,
    key: &str,
    value: &str,
    updated_at: &str,
) -> Result<(), CliplyError> {
    connection.execute(
        "INSERT INTO sync_state (key, value, updated_at)
         VALUES (?1, ?2, ?3)
         ON CONFLICT(key) DO UPDATE SET
           value = excluded.value,
           updated_at = excluded.updated_at",
        params![key, value, updated_at],
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
    use super::{
        abandon_sync_queue_at, load_sync_export_checkpoint, load_sync_queue_status,
        mark_sync_exported, AUTO_SYNC_ENABLED_KEY,
    };
    use rusqlite::{params, Connection};

    #[test]
    fn abandoning_queue_preserves_rows_and_records_counts() {
        let mut connection = setup_connection();
        insert_item(
            &connection,
            "live",
            false,
            "pending",
            "2026-08-01T10:00:00Z",
        );
        insert_item(
            &connection,
            "tombstone",
            true,
            "pending",
            "2026-07-01T10:00:00Z",
        );
        connection
            .execute(
                "INSERT INTO sync_events (
                    id, item_id, event_type, created_at, synced_at, abandoned_at
                 ) VALUES ('event-1', 'live', 'item_updated', '2026-08-01T10:00:00Z', NULL, NULL)",
                [],
            )
            .expect("event should insert");
        connection
            .execute(
                "INSERT INTO sync_blobs (
                    id, item_id, blob_type, remote_path, hash, sync_status, created_at
                 ) VALUES ('blob-1', 'live', 'preview', '/blob', 'hash', 'pending', '2026-08-01T10:00:00Z')",
                [],
            )
            .expect("blob should insert");
        set_state(&connection, AUTO_SYNC_ENABLED_KEY, "true");

        let result = abandon_sync_queue_at(&mut connection, "2026-08-14T08:00:00Z")
            .expect("queue should be abandoned");

        assert_eq!(result.item_count, 1);
        assert_eq!(result.tombstone_count, 1);
        assert_eq!(result.event_count, 1);
        assert_eq!(result.blob_count, 1);
        assert_eq!(row_count(&connection, "clipboard_items"), 2);
        assert_eq!(item_status(&connection, "live"), "abandoned");
        assert_eq!(item_status(&connection, "tombstone"), "abandoned");
        assert_eq!(blob_status(&connection, "blob-1"), "abandoned");
        assert_eq!(
            event_abandoned_at(&connection, "event-1"),
            Some("2026-08-14T08:00:00Z".to_string())
        );

        let status = load_sync_queue_status(&connection).expect("status should load");
        assert_eq!(status.pending_change_count(), 0);
        assert!(status.full_export_required);
        assert_eq!(status.last_abandoned_item_count, 1);
        assert_eq!(status.last_abandoned_tombstone_count, 1);
        assert_eq!(status.last_abandoned_event_count, 1);
        assert_eq!(status.last_abandoned_blob_count, 1);
        assert_eq!(
            state_value(&connection, AUTO_SYNC_ENABLED_KEY),
            Some("false".to_string())
        );
    }

    #[test]
    fn export_checkpoint_does_not_mark_newer_changes_synced() {
        let mut connection = setup_connection();
        insert_item(
            &connection,
            "before",
            false,
            "pending",
            "2026-08-14T08:00:00.100Z",
        );
        insert_item(
            &connection,
            "after",
            false,
            "pending",
            "2026-08-14T08:00:00.900Z",
        );
        connection
            .execute_batch(
                "INSERT INTO sync_events (
                    id, item_id, event_type, created_at, synced_at, abandoned_at
                 ) VALUES
                   ('event-before', 'before', 'item_updated', '2026-08-14T08:00:00.100Z', NULL, NULL),
                   ('event-after', 'after', 'item_updated', '2026-08-14T08:00:00.900Z', NULL, NULL);",
            )
            .expect("events should insert");
        let checkpoint = load_sync_export_checkpoint(&connection).expect("checkpoint should load");

        mark_sync_exported(&mut connection, "2026-08-14T08:00:00.500Z", &checkpoint)
            .expect("export should be marked");

        assert_eq!(item_status(&connection, "before"), "synced");
        assert_eq!(item_status(&connection, "after"), "pending");
        assert!(event_synced_at(&connection, "event-before").is_some());
        assert!(event_synced_at(&connection, "event-after").is_none());
    }

    #[test]
    fn full_baseline_only_merges_matching_abandonment_checkpoint() {
        let mut connection = setup_connection();
        insert_item(
            &connection,
            "old-abandoned",
            true,
            "abandoned",
            "2026-07-01T08:00:00Z",
        );
        set_state(&connection, "remote_sync_full_export_required", "true");
        set_state(
            &connection,
            "remote_sync_last_queue_abandoned_at",
            "2026-08-14T08:00:00Z",
        );
        let checkpoint = load_sync_export_checkpoint(&connection).expect("checkpoint should load");

        set_state(
            &connection,
            "remote_sync_last_queue_abandoned_at",
            "2026-08-14T08:05:00Z",
        );
        let stale_result = mark_sync_exported(&mut connection, "2026-08-14T08:02:00Z", &checkpoint)
            .expect("stale baseline should finish safely");

        assert!(!stale_result.full_baseline_applied);
        assert_eq!(item_status(&connection, "old-abandoned"), "abandoned");
        assert_eq!(
            state_value(&connection, "remote_sync_full_export_required"),
            Some("true".to_string())
        );

        let current_checkpoint =
            load_sync_export_checkpoint(&connection).expect("current checkpoint should load");
        let current_result =
            mark_sync_exported(&mut connection, "2026-08-14T08:06:00Z", &current_checkpoint)
                .expect("current baseline should merge abandoned rows");

        assert!(current_result.full_baseline_applied);
        assert_eq!(current_result.remaining_abandoned_count, 0);
        assert_eq!(item_status(&connection, "old-abandoned"), "synced");
        assert_eq!(
            state_value(&connection, "remote_sync_full_export_required"),
            Some("false".to_string())
        );
    }

    fn setup_connection() -> Connection {
        let connection = Connection::open_in_memory().expect("in-memory sqlite should open");
        connection
            .execute_batch(
                "
                CREATE TABLE clipboard_items (
                  id TEXT PRIMARY KEY,
                  is_deleted INTEGER DEFAULT 0,
                  updated_at TEXT NOT NULL,
                  sync_status TEXT DEFAULT 'pending',
                  last_synced_at TEXT NULL
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
                CREATE TABLE sync_blobs (
                  id TEXT PRIMARY KEY,
                  item_id TEXT NOT NULL,
                  blob_type TEXT NOT NULL,
                  remote_path TEXT,
                  hash TEXT NOT NULL,
                  sync_status TEXT DEFAULT 'pending',
                  created_at TEXT NOT NULL,
                  uploaded_at TEXT NULL,
                  deleted_at TEXT NULL
                );
                CREATE TABLE sync_state (
                  key TEXT PRIMARY KEY,
                  value TEXT NOT NULL,
                  updated_at TEXT NOT NULL
                );
                ",
            )
            .expect("sync queue schema should initialize");
        connection
    }

    fn insert_item(
        connection: &Connection,
        id: &str,
        deleted: bool,
        status: &str,
        updated_at: &str,
    ) {
        connection
            .execute(
                "INSERT INTO clipboard_items (id, is_deleted, updated_at, sync_status)
                 VALUES (?1, ?2, ?3, ?4)",
                params![id, if deleted { 1 } else { 0 }, updated_at, status],
            )
            .expect("item should insert");
    }

    fn set_state(connection: &Connection, key: &str, value: &str) {
        connection
            .execute(
                "INSERT INTO sync_state (key, value, updated_at)
                 VALUES (?1, ?2, '2026-08-14T08:00:00Z')
                 ON CONFLICT(key) DO UPDATE SET value = excluded.value",
                params![key, value],
            )
            .expect("state should save");
    }

    fn state_value(connection: &Connection, key: &str) -> Option<String> {
        connection
            .query_row(
                "SELECT value FROM sync_state WHERE key = ?1",
                params![key],
                |row| row.get(0),
            )
            .ok()
    }

    fn item_status(connection: &Connection, id: &str) -> String {
        connection
            .query_row(
                "SELECT sync_status FROM clipboard_items WHERE id = ?1",
                params![id],
                |row| row.get(0),
            )
            .expect("item status should load")
    }

    fn blob_status(connection: &Connection, id: &str) -> String {
        connection
            .query_row(
                "SELECT sync_status FROM sync_blobs WHERE id = ?1",
                params![id],
                |row| row.get(0),
            )
            .expect("blob status should load")
    }

    fn event_abandoned_at(connection: &Connection, id: &str) -> Option<String> {
        connection
            .query_row(
                "SELECT abandoned_at FROM sync_events WHERE id = ?1",
                params![id],
                |row| row.get(0),
            )
            .expect("event abandonment should load")
    }

    fn event_synced_at(connection: &Connection, id: &str) -> Option<String> {
        connection
            .query_row(
                "SELECT synced_at FROM sync_events WHERE id = ?1",
                params![id],
                |row| row.get(0),
            )
            .expect("event sync time should load")
    }

    fn row_count(connection: &Connection, table: &str) -> i64 {
        connection
            .query_row(&format!("SELECT COUNT(*) FROM {table}"), [], |row| {
                row.get(0)
            })
            .expect("row count should load")
    }
}
