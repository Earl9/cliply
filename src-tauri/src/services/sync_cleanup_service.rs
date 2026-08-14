use crate::error::CliplyError;
use crate::services::{database_service, sync_package_service};
use rusqlite::{params, Connection, TransactionBehavior};
use tauri::AppHandle;
use time::OffsetDateTime;

const VACUUM_DELETED_ROW_THRESHOLD: usize = 2_000;
const VACUUM_FREE_BYTES_THRESHOLD: u64 = 8 * 1024 * 1024;
const VACUUM_FREE_PAGE_PERCENT_THRESHOLD: u64 = 20;

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct SyncCleanupResult {
    pub deleted_tombstones: usize,
    pub deleted_format_rows: usize,
    pub deleted_tag_rows: usize,
    pub deleted_blob_rows: usize,
    pub deleted_event_rows: usize,
    pub total_deleted_rows: usize,
    pub page_count: u64,
    pub freelist_pages: u64,
    pub page_size: u64,
    pub vacuumed: bool,
}

pub fn run_sync_cleanup(app: &AppHandle) -> Result<SyncCleanupResult, CliplyError> {
    let mut connection = database_service::connect(app)?;
    let cutoff = cleanup_cutoff_timestamp()?;
    cleanup_sync_data_at(&mut connection, &cutoff)
}

pub fn cleanup_sync_data_at(
    connection: &mut Connection,
    cutoff: &str,
) -> Result<SyncCleanupResult, CliplyError> {
    let transaction = connection.transaction_with_behavior(TransactionBehavior::Immediate)?;
    transaction.execute_batch(
        "CREATE TEMP TABLE IF NOT EXISTS cliply_sync_cleanup_candidates (
           item_id TEXT PRIMARY KEY
         );
         DELETE FROM cliply_sync_cleanup_candidates;",
    )?;
    transaction.execute(
        "INSERT OR IGNORE INTO cliply_sync_cleanup_candidates (item_id)
         SELECT item.id
         FROM clipboard_items item
         WHERE item.is_deleted = 1
           AND item.sync_status = 'synced'
           AND item.deleted_at IS NOT NULL
           AND julianday(item.deleted_at) < julianday(?1)
           AND NOT EXISTS (
             SELECT 1
             FROM sync_events event
             WHERE event.item_id = item.id
               AND event.synced_at IS NULL
           )
           AND NOT EXISTS (
             SELECT 1
             FROM sync_blobs blob
             WHERE blob.item_id = item.id
               AND COALESCE(blob.sync_status, 'pending') <> 'synced'
           )",
        params![cutoff],
    )?;

    let deleted_tombstones = query_count(
        &transaction,
        "SELECT COUNT(*) FROM cliply_sync_cleanup_candidates",
    )?;
    let deleted_fts_rows = transaction.execute(
        "DELETE FROM clipboard_items_fts
         WHERE item_id IN (SELECT item_id FROM cliply_sync_cleanup_candidates)",
        [],
    )?;
    let deleted_format_rows = transaction.execute(
        "DELETE FROM clipboard_formats
         WHERE item_id IN (SELECT item_id FROM cliply_sync_cleanup_candidates)",
        [],
    )?;
    let deleted_tag_rows = transaction.execute(
        "DELETE FROM clipboard_tags
         WHERE item_id IN (SELECT item_id FROM cliply_sync_cleanup_candidates)",
        [],
    )?;
    let deleted_blob_rows = transaction.execute(
        "DELETE FROM sync_blobs
         WHERE item_id IN (SELECT item_id FROM cliply_sync_cleanup_candidates)",
        [],
    )?;
    let candidate_event_rows = transaction.execute(
        "DELETE FROM sync_events
         WHERE item_id IN (SELECT item_id FROM cliply_sync_cleanup_candidates)",
        [],
    )?;
    let removed_items = transaction.execute(
        "DELETE FROM clipboard_items
         WHERE id IN (SELECT item_id FROM cliply_sync_cleanup_candidates)",
        [],
    )?;
    let old_synced_event_rows = transaction.execute(
        "DELETE FROM sync_events
         WHERE synced_at IS NOT NULL
           AND julianday(created_at) < julianday(?1)",
        params![cutoff],
    )?;
    transaction.commit()?;

    let deleted_event_rows = candidate_event_rows + old_synced_event_rows;
    let total_deleted_rows = deleted_fts_rows
        + deleted_format_rows
        + deleted_tag_rows
        + deleted_blob_rows
        + deleted_event_rows
        + removed_items;

    let (page_count, freelist_pages, page_size) = if total_deleted_rows > 0 {
        database_space_metrics(connection)?
    } else {
        (0, 0, 0)
    };
    let vacuumed = total_deleted_rows > 0
        && should_vacuum(total_deleted_rows, page_count, freelist_pages, page_size);
    if vacuumed {
        connection.execute_batch("VACUUM")?;
    }

    Ok(SyncCleanupResult {
        deleted_tombstones,
        deleted_format_rows,
        deleted_tag_rows,
        deleted_blob_rows,
        deleted_event_rows,
        total_deleted_rows,
        page_count,
        freelist_pages,
        page_size,
        vacuumed,
    })
}

pub(crate) fn should_vacuum(
    deleted_rows: usize,
    page_count: u64,
    freelist_pages: u64,
    page_size: u64,
) -> bool {
    if deleted_rows >= VACUUM_DELETED_ROW_THRESHOLD {
        return true;
    }
    if page_count == 0 || freelist_pages == 0 || page_size == 0 {
        return false;
    }

    let free_bytes = freelist_pages.saturating_mul(page_size);
    let free_percent = freelist_pages.saturating_mul(100) / page_count;
    free_bytes >= VACUUM_FREE_BYTES_THRESHOLD && free_percent >= VACUUM_FREE_PAGE_PERCENT_THRESHOLD
}

fn database_space_metrics(connection: &Connection) -> Result<(u64, u64, u64), CliplyError> {
    let page_count = pragma_u64(connection, "page_count")?;
    let freelist_pages = pragma_u64(connection, "freelist_count")?;
    let page_size = pragma_u64(connection, "page_size")?;
    Ok((page_count, freelist_pages, page_size))
}

fn pragma_u64(connection: &Connection, pragma: &str) -> Result<u64, CliplyError> {
    let value: i64 = connection.query_row(&format!("PRAGMA {pragma}"), [], |row| row.get(0))?;
    Ok(u64::try_from(value.max(0)).unwrap_or(u64::MAX))
}

fn query_count(connection: &Connection, sql: &str) -> Result<usize, CliplyError> {
    let count: i64 = connection.query_row(sql, [], |row| row.get(0))?;
    Ok(usize::try_from(count.max(0)).unwrap_or(usize::MAX))
}

fn cleanup_cutoff_timestamp() -> Result<String, CliplyError> {
    (OffsetDateTime::now_utc()
        - time::Duration::days(sync_package_service::SYNC_EXPORT_WINDOW_DAYS))
    .format(&time::format_description::well_known::Rfc3339)
    .map_err(|error| CliplyError::StorageUnavailable(error.to_string()))
}

#[cfg(test)]
mod tests {
    use super::{cleanup_sync_data_at, should_vacuum};
    use rusqlite::{params, Connection};

    #[test]
    fn cleanup_removes_only_old_fully_synced_tombstones_and_related_rows() {
        let mut connection = setup_connection();
        insert_item(&connection, "eligible", "synced", "2026-06-01T00:00:00Z");
        insert_item(&connection, "recent", "synced", "2026-08-10T00:00:00Z");
        insert_item(&connection, "pending", "pending", "2026-06-01T00:00:00Z");
        insert_item(
            &connection,
            "abandoned",
            "abandoned",
            "2026-06-01T00:00:00Z",
        );
        insert_item(
            &connection,
            "blocked-event",
            "synced",
            "2026-06-01T00:00:00Z",
        );
        insert_item(
            &connection,
            "blocked-blob",
            "synced",
            "2026-06-01T00:00:00Z",
        );

        insert_related_rows(&connection, "eligible", "synced", true);
        insert_related_rows(&connection, "recent", "synced", true);
        insert_related_rows(&connection, "pending", "pending", false);
        insert_related_rows(&connection, "abandoned", "abandoned", false);
        insert_related_rows(&connection, "blocked-event", "synced", false);
        insert_related_rows(&connection, "blocked-blob", "pending", true);
        connection
            .execute(
                "INSERT INTO sync_events (
                    id, item_id, event_type, created_at, synced_at, abandoned_at
                 ) VALUES (
                    'blocked-event-pending', 'blocked-event', 'item_deleted',
                    '2026-06-01T00:00:00Z', NULL, NULL
                 )",
                [],
            )
            .expect("blocking event should insert");
        connection
            .execute_batch(
                "INSERT INTO sync_events (
                    id, item_id, event_type, created_at, synced_at, abandoned_at
                 ) VALUES
                   ('old-global-synced', NULL, 'device_seen', '2026-06-01T00:00:00Z', '2026-06-02T00:00:00Z', NULL),
                   ('old-global-pending', NULL, 'device_seen', '2026-06-01T00:00:00Z', NULL, NULL);",
            )
            .expect("global events should insert");

        let result = cleanup_sync_data_at(&mut connection, "2026-07-15T00:00:00Z")
            .expect("cleanup should succeed");

        assert_eq!(result.deleted_tombstones, 1);
        assert!(!result.vacuumed);
        assert!(!item_exists(&connection, "eligible"));
        assert_eq!(
            related_count(&connection, "clipboard_formats", "eligible"),
            0
        );
        assert_eq!(related_count(&connection, "clipboard_tags", "eligible"), 0);
        assert_eq!(related_count(&connection, "sync_blobs", "eligible"), 0);
        assert_eq!(related_count(&connection, "sync_events", "eligible"), 0);
        assert_eq!(
            related_count(&connection, "clipboard_items_fts", "eligible"),
            0
        );
        for preserved in [
            "recent",
            "pending",
            "abandoned",
            "blocked-event",
            "blocked-blob",
        ] {
            assert!(
                item_exists(&connection, preserved),
                "{preserved} should remain"
            );
        }
        assert!(!event_exists(&connection, "old-global-synced"));
        assert!(event_exists(&connection, "old-global-pending"));
    }

    #[test]
    fn vacuum_requires_deleted_row_or_free_space_threshold() {
        assert!(!should_vacuum(1_999, 10_000, 1_999, 4_096));
        assert!(should_vacuum(2_000, 1, 0, 4_096));
        assert!(should_vacuum(10, 10_240, 2_048, 4_096));
        assert!(!should_vacuum(10, 10_240, 2_047, 4_096));
        assert!(!should_vacuum(10, 0, 0, 0));
    }

    fn setup_connection() -> Connection {
        let connection = Connection::open_in_memory().expect("in-memory sqlite should open");
        connection
            .execute_batch(
                "
                CREATE TABLE clipboard_items (
                  id TEXT PRIMARY KEY,
                  is_deleted INTEGER DEFAULT 0,
                  deleted_at TEXT NULL,
                  sync_status TEXT DEFAULT 'pending'
                );
                CREATE TABLE clipboard_items_fts (
                  item_id TEXT PRIMARY KEY,
                  title TEXT
                );
                CREATE TABLE clipboard_formats (
                  id TEXT PRIMARY KEY,
                  item_id TEXT NOT NULL
                );
                CREATE TABLE clipboard_tags (
                  item_id TEXT NOT NULL,
                  tag TEXT NOT NULL,
                  PRIMARY KEY (item_id, tag)
                );
                CREATE TABLE sync_events (
                  id TEXT PRIMARY KEY,
                  item_id TEXT,
                  event_type TEXT NOT NULL,
                  created_at TEXT NOT NULL,
                  synced_at TEXT NULL,
                  abandoned_at TEXT NULL
                );
                CREATE TABLE sync_blobs (
                  id TEXT PRIMARY KEY,
                  item_id TEXT NOT NULL,
                  sync_status TEXT DEFAULT 'pending'
                );
                ",
            )
            .expect("cleanup schema should initialize");
        connection
    }

    fn insert_item(connection: &Connection, id: &str, status: &str, deleted_at: &str) {
        connection
            .execute(
                "INSERT INTO clipboard_items (id, is_deleted, deleted_at, sync_status)
                 VALUES (?1, 1, ?2, ?3)",
                params![id, deleted_at, status],
            )
            .expect("item should insert");
    }

    fn insert_related_rows(
        connection: &Connection,
        item_id: &str,
        blob_status: &str,
        event_synced: bool,
    ) {
        connection
            .execute(
                "INSERT INTO clipboard_items_fts (item_id, title) VALUES (?1, ?1)",
                params![item_id],
            )
            .expect("fts row should insert");
        connection
            .execute(
                "INSERT INTO clipboard_formats (id, item_id) VALUES (?1, ?2)",
                params![format!("format-{item_id}"), item_id],
            )
            .expect("format should insert");
        connection
            .execute(
                "INSERT INTO clipboard_tags (item_id, tag) VALUES (?1, 'tag')",
                params![item_id],
            )
            .expect("tag should insert");
        connection
            .execute(
                "INSERT INTO sync_blobs (id, item_id, sync_status) VALUES (?1, ?2, ?3)",
                params![format!("blob-{item_id}"), item_id, blob_status],
            )
            .expect("blob should insert");
        connection
            .execute(
                "INSERT INTO sync_events (
                    id, item_id, event_type, created_at, synced_at, abandoned_at
                 ) VALUES (?1, ?2, 'item_deleted', '2026-06-01T00:00:00Z', ?3, NULL)",
                params![
                    format!("event-{item_id}"),
                    item_id,
                    event_synced.then_some("2026-06-02T00:00:00Z")
                ],
            )
            .expect("event should insert");
    }

    fn item_exists(connection: &Connection, id: &str) -> bool {
        connection
            .query_row(
                "SELECT COUNT(*) FROM clipboard_items WHERE id = ?1",
                params![id],
                |row| row.get::<_, i64>(0),
            )
            .expect("item count should load")
            > 0
    }

    fn related_count(connection: &Connection, table: &str, item_id: &str) -> i64 {
        connection
            .query_row(
                &format!("SELECT COUNT(*) FROM {table} WHERE item_id = ?1"),
                params![item_id],
                |row| row.get(0),
            )
            .expect("related count should load")
    }

    fn event_exists(connection: &Connection, id: &str) -> bool {
        connection
            .query_row(
                "SELECT COUNT(*) FROM sync_events WHERE id = ?1",
                params![id],
                |row| row.get::<_, i64>(0),
            )
            .expect("event count should load")
            > 0
    }
}
