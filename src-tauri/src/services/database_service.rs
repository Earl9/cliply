use crate::error::CliplyError;
use crate::services::sync_service;
use crate::{db, logger};
use rusqlite::{params, Connection};
use tauri::AppHandle;

const INIT_MIGRATION: &str = include_str!("../db/migrations/001_init.sql");
const FTS_MIGRATION: &str = include_str!("../db/migrations/002_fts.sql");
const SYNC_MIGRATION: &str = include_str!("../db/migrations/003_sync.sql");
const SYNC_BLOBS_MIGRATION: &str = include_str!("../db/migrations/004_sync_blobs.sql");
const PERF_INDEX_MIGRATION: &str = include_str!("../db/migrations/005_perf_indexes.sql");
const SYNC_QUEUE_CLEANUP_MIGRATION: &str =
    include_str!("../db/migrations/006_sync_queue_cleanup.sql");
const DATABASE_OPTIMIZE_STATE_KEY: &str = "database_last_optimized_at";
const DATABASE_OPTIMIZE_INTERVAL_DAYS: i64 = 7;

pub fn initialize(app: &AppHandle) -> Result<(), CliplyError> {
    let connection = connect(app)?;
    connection.execute_batch(INIT_MIGRATION)?;
    connection.execute_batch(FTS_MIGRATION)?;
    apply_sync_migration(&connection)?;
    apply_sync_blobs_migration(&connection)?;
    connection.execute_batch(PERF_INDEX_MIGRATION)?;
    apply_sync_queue_cleanup_migration(&connection)?;
    let device = sync_service::initialize_device(&connection)?;
    logger::info(
        app,
        "sync_device_initialized",
        format!("device_id={}", device.id),
    );
    let removed_legacy_items = hide_legacy_privacy_placeholder_items(&connection)?;
    if removed_legacy_items > 0 {
        logger::info(
            app,
            "legacy_privacy_placeholders_hidden",
            format!("count={removed_legacy_items}"),
        );
    }
    seed_mock_data(&connection)?;
    migrate_default_theme(&connection)?;
    if optimize_database_if_due(&connection)? {
        logger::info(app, "database_optimize", "completed");
    }
    Ok(())
}

fn optimize_database_if_due(connection: &Connection) -> Result<bool, CliplyError> {
    let optimize_due = connection.query_row(
        "SELECT CASE
           WHEN EXISTS (
             SELECT 1
             FROM sync_state
             WHERE key = ?1
               AND julianday(value) IS NOT NULL
               AND julianday('now') - julianday(value) < ?2
           ) THEN 0
           ELSE 1
         END",
        params![DATABASE_OPTIMIZE_STATE_KEY, DATABASE_OPTIMIZE_INTERVAL_DAYS],
        |row| Ok(row.get::<_, i64>(0)? == 1),
    )?;
    if !optimize_due {
        return Ok(false);
    }

    connection.execute_batch("PRAGMA optimize;")?;
    connection.execute(
        "INSERT INTO sync_state (key, value, updated_at)
         VALUES (?1, strftime('%Y-%m-%dT%H:%M:%fZ', 'now'), strftime('%Y-%m-%dT%H:%M:%fZ', 'now'))
         ON CONFLICT(key) DO UPDATE SET
           value = excluded.value,
           updated_at = excluded.updated_at",
        params![DATABASE_OPTIMIZE_STATE_KEY],
    )?;
    Ok(true)
}

/// One-time migrations that move installs still sitting on a previous
/// *untouched default* onto the current one. Each step has its own marker so a
/// user who deliberately picks a colour afterwards is never overridden again.
fn migrate_default_theme(connection: &Connection) -> Result<(), CliplyError> {
    // purple-default/#6D4CFF -> system-blue/#0067C0
    migrate_default_accent(
        connection,
        "theme_default_migrated_v1",
        Some("\"purple-default\""),
        "\"#6D4CFF\"",
        Some("\"system-blue\""),
        "\"#0067C0\"",
    )?;
    // The system-blue theme itself moved to the lighter #1F74CC accent.
    migrate_default_accent(
        connection,
        "theme_default_migrated_v2",
        Some("\"system-blue\""),
        "\"#0067C0\"",
        None,
        "\"#1F74CC\"",
    )?;
    // The youthful palette raises the untouched default to a clearer cobalt.
    migrate_default_accent(
        connection,
        "theme_default_migrated_v3",
        Some("\"system-blue\""),
        "\"#1F74CC\"",
        None,
        "\"#2F69FA\"",
    )?;
    // The brand-led palette replaces the untouched blue default with coral.
    migrate_default_accent(
        connection,
        "theme_default_migrated_v4",
        Some("\"system-blue\""),
        "\"#2F69FA\"",
        Some("\"coral-pulse\""),
        "\"#FF6257\"",
    )?;
    Ok(())
}

fn migrate_default_accent(
    connection: &Connection,
    marker_key: &str,
    expected_theme: Option<&str>,
    expected_accent: &str,
    next_theme: Option<&str>,
    next_accent: &str,
) -> Result<(), CliplyError> {
    use rusqlite::OptionalExtension;

    let migrated: i64 = connection.query_row(
        "SELECT COUNT(*) FROM settings WHERE key = ?1",
        params![marker_key],
        |row| row.get(0),
    )?;
    if migrated > 0 {
        return Ok(());
    }

    let theme_name: Option<String> = connection
        .query_row(
            "SELECT value FROM settings WHERE key = 'theme_name'",
            [],
            |row| row.get(0),
        )
        .optional()?;
    let accent_color: Option<String> = connection
        .query_row(
            "SELECT value FROM settings WHERE key = 'accent_color'",
            [],
            |row| row.get(0),
        )
        .optional()?;

    if theme_name.as_deref() == expected_theme && accent_color.as_deref() == Some(expected_accent) {
        if let Some(next_theme) = next_theme {
            connection.execute(
                "UPDATE settings SET value = ?1 WHERE key = 'theme_name'",
                params![next_theme],
            )?;
        }
        connection.execute(
            "UPDATE settings SET value = ?1 WHERE key = 'accent_color'",
            params![next_accent],
        )?;
    }

    connection.execute(
        "INSERT OR IGNORE INTO settings (key, value, updated_at)
         VALUES (?1, 'true', datetime('now'))",
        params![marker_key],
    )?;
    Ok(())
}

pub fn connect(app: &AppHandle) -> Result<Connection, CliplyError> {
    let path = db::database_path(app)?;
    let connection = Connection::open(path)?;
    configure_connection(&connection)?;
    Ok(connection)
}

/// Shared connection setup: WAL keeps readers unblocked while the clipboard
/// worker or sync thread writes, and busy_timeout absorbs short lock windows
/// instead of surfacing "database is locked" to the user.
fn configure_connection(connection: &Connection) -> Result<(), CliplyError> {
    connection.pragma_update(None, "foreign_keys", "ON")?;
    connection.pragma_update(None, "journal_mode", "WAL")?;
    connection.pragma_update(None, "synchronous", "NORMAL")?;
    connection.pragma_update(None, "busy_timeout", "5000")?;
    connection.pragma_update(None, "cache_size", "-8000")?;
    Ok(())
}

fn seed_mock_data(connection: &Connection) -> Result<(), CliplyError> {
    let count: i64 =
        connection.query_row("SELECT COUNT(*) FROM clipboard_items", [], |row| row.get(0))?;

    if count > 0 {
        return Ok(());
    }

    let items = [
        SeedItem {
            id: "seed-code-auth",
            item_type: "code",
            title: "Session guard",
            preview_text: "const user = await getProfile(session.userId);",
            normalized_text: "const user = await getProfile(session.userId);\n\nif (!user?.enabled) {\n  return createEmptySession();\n}\n\nreturn createSession(user);",
            source_app: "Visual Studio Code",
            source_window: "auth/session.ts",
            hash: "seed-code-auth-hash",
            size_bytes: 148,
            is_pinned: true,
            copied_at: "2026-05-04T10:42:18+08:00",
            tags: &["typescript", "auth"],
        },
        SeedItem {
            id: "seed-link-tauri",
            item_type: "link",
            title: "Tauri v2 repository",
            preview_text: "https://github.com/tauri-apps/tauri",
            normalized_text: "https://github.com/tauri-apps/tauri",
            source_app: "Chrome",
            source_window: "GitHub",
            hash: "seed-link-tauri-hash",
            size_bytes: 37,
            is_pinned: false,
            copied_at: "2026-05-04T10:41:03+08:00",
            tags: &["tauri", "rust"],
        },
        SeedItem {
            id: "seed-text-principles",
            item_type: "text",
            title: "MVP principles",
            preview_text: "Keep the Windows MVP runnable and keep platform adapters clear.",
            normalized_text: "Keep the Windows MVP runnable and keep platform adapters clear. Do not add cloud services or accounts in v1.",
            source_app: "Notepad",
            source_window: "Cliply notes.txt",
            hash: "seed-text-principles-hash",
            size_bytes: 105,
            is_pinned: false,
            copied_at: "2026-05-04T10:34:00+08:00",
            tags: &["mvp"],
        },
        SeedItem {
            id: "seed-image-window",
            item_type: "image",
            title: "Main window mockup",
            preview_text: "Screenshot 1160 x 760",
            normalized_text: "",
            source_app: "Snipping Tool",
            source_window: "Screen snip",
            hash: "seed-image-window-hash",
            size_bytes: 421888,
            is_pinned: true,
            copied_at: "2026-05-04T10:24:12+08:00",
            tags: &["ui", "mockup"],
        },
    ];

    for item in items {
        insert_seed_item(connection, &item)?;
    }

    Ok(())
}

fn insert_seed_item(connection: &Connection, item: &SeedItem) -> Result<(), CliplyError> {
    let now = item.copied_at;
    let device_id = sync_service::current_device_id(connection)?;
    let sync_id = format!("sync-{}", item.id);

    connection.execute(
        "INSERT INTO clipboard_items (
            id, type, title, preview_text, normalized_text, source_app, source_window,
            hash, size_bytes, is_pinned, copied_at, created_at, updated_at, used_count,
            sync_id, device_id, revision, deleted_at, sync_status, last_synced_at
        ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, 0,
                  ?14, ?15, 1, NULL, 'pending', NULL)",
        params![
            item.id,
            item.item_type,
            item.title,
            item.preview_text,
            item.normalized_text,
            item.source_app,
            item.source_window,
            item.hash,
            item.size_bytes,
            if item.is_pinned { 1 } else { 0 },
            item.copied_at,
            now,
            now,
            sync_id,
            device_id
        ],
    )?;

    connection.execute(
        "INSERT INTO clipboard_formats (
            id, item_id, format_name, mime_type, data_kind, data_text, size_bytes, priority, created_at
        ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, 0, ?8)",
        params![
            format!("{}-format-primary", item.id),
            item.id,
            if item.item_type == "image" {
                "image/png"
            } else {
                "text/plain"
            },
            if item.item_type == "image" {
                "image/png"
            } else {
                "text/plain"
            },
            if item.item_type == "image" {
                "image_file"
            } else {
                "text"
            },
            if item.item_type == "image" {
                ""
            } else {
                item.normalized_text
            },
            item.size_bytes,
            now
        ],
    )?;

    for tag in item.tags {
        connection.execute(
            "INSERT INTO clipboard_tags (item_id, tag, created_at) VALUES (?1, ?2, ?3)",
            params![item.id, tag, now],
        )?;
    }

    connection.execute(
        "INSERT INTO clipboard_items_fts (
            item_id, title, preview_text, normalized_text, source_app
        ) VALUES (?1, ?2, ?3, ?4, ?5)",
        params![
            item.id,
            item.title,
            item.preview_text,
            item.normalized_text,
            item.source_app
        ],
    )?;

    Ok(())
}

fn apply_sync_migration(connection: &Connection) -> Result<(), CliplyError> {
    add_column_if_missing(connection, "clipboard_items", "sync_id", "TEXT")?;
    add_column_if_missing(connection, "clipboard_items", "device_id", "TEXT")?;
    add_column_if_missing(
        connection,
        "clipboard_items",
        "revision",
        "INTEGER DEFAULT 1",
    )?;
    add_column_if_missing(connection, "clipboard_items", "deleted_at", "TEXT NULL")?;
    add_column_if_missing(
        connection,
        "clipboard_items",
        "sync_status",
        "TEXT DEFAULT 'pending'",
    )?;
    add_column_if_missing(connection, "clipboard_items", "last_synced_at", "TEXT NULL")?;

    connection.execute_batch(SYNC_MIGRATION)?;

    Ok(())
}

fn apply_sync_blobs_migration(connection: &Connection) -> Result<(), CliplyError> {
    connection.execute_batch(SYNC_BLOBS_MIGRATION)?;
    Ok(())
}

fn apply_sync_queue_cleanup_migration(connection: &Connection) -> Result<(), CliplyError> {
    add_column_if_missing(connection, "sync_events", "abandoned_at", "TEXT NULL")?;
    connection.execute_batch(SYNC_QUEUE_CLEANUP_MIGRATION)?;
    Ok(())
}

fn hide_legacy_privacy_placeholder_items(connection: &Connection) -> Result<usize, CliplyError> {
    let changed = connection.execute(
        "UPDATE clipboard_items
         SET is_deleted = 1,
             deleted_at = COALESCE(deleted_at, datetime('now')),
             sync_status = CASE
               WHEN sync_status IS NULL OR sync_status = 'synced' THEN 'pending'
               ELSE sync_status
             END
         WHERE is_deleted = 0
           AND deleted_at IS NULL
           AND (
             COALESCE(title, '') LIKE '已隐藏敏感内容%'
             OR COALESCE(preview_text, '') LIKE '已隐藏敏感内容%'
             OR COALESCE(preview_text, '') LIKE '已隐藏疑似验证码%'
             OR COALESCE(source_app, '') = 'Privacy'
             OR COALESCE(source_app, '') = '隐私'
           )",
        [],
    )?;

    if changed > 0 {
        let _ = connection.execute(
            "DELETE FROM clipboard_items_fts
             WHERE item_id IN (
               SELECT id
               FROM clipboard_items
               WHERE is_deleted = 1
                 AND (
                   COALESCE(title, '') LIKE '已隐藏敏感内容%'
                   OR COALESCE(preview_text, '') LIKE '已隐藏敏感内容%'
                   OR COALESCE(preview_text, '') LIKE '已隐藏疑似验证码%'
                   OR COALESCE(source_app, '') = 'Privacy'
                   OR COALESCE(source_app, '') = '隐私'
                 )
             )",
            [],
        );
    }

    Ok(changed)
}

fn add_column_if_missing(
    connection: &Connection,
    table_name: &str,
    column_name: &str,
    column_definition: &str,
) -> Result<(), CliplyError> {
    let exists = connection
        .prepare(&format!("PRAGMA table_info({table_name})"))?
        .query_map([], |row| row.get::<_, String>(1))?
        .collect::<Result<Vec<_>, _>>()?
        .iter()
        .any(|existing| existing == column_name);

    if !exists {
        connection.execute(
            &format!("ALTER TABLE {table_name} ADD COLUMN {column_name} {column_definition}"),
            [],
        )?;
    }

    Ok(())
}

struct SeedItem<'a> {
    id: &'a str,
    item_type: &'a str,
    title: &'a str,
    preview_text: &'a str,
    normalized_text: &'a str,
    source_app: &'a str,
    source_window: &'a str,
    hash: &'a str,
    size_bytes: i64,
    is_pinned: bool,
    copied_at: &'a str,
    tags: &'a [&'a str],
}

#[cfg(test)]
mod tests {
    use super::{
        apply_sync_blobs_migration, apply_sync_migration, apply_sync_queue_cleanup_migration,
        hide_legacy_privacy_placeholder_items, optimize_database_if_due, sync_service,
    };
    use rusqlite::{params, Connection};

    #[test]
    fn sync_migration_adds_columns_and_device_identity() {
        let connection = Connection::open_in_memory().expect("in-memory sqlite should open");
        connection
            .execute_batch(
                "
                CREATE TABLE clipboard_items (
                  id TEXT PRIMARY KEY,
                  type TEXT NOT NULL,
                  hash TEXT NOT NULL,
                  is_deleted INTEGER DEFAULT 0,
                  copied_at TEXT NOT NULL,
                  created_at TEXT NOT NULL,
                  updated_at TEXT NOT NULL
                );

                INSERT INTO clipboard_items (
                  id, type, hash, is_deleted, copied_at, created_at, updated_at
                ) VALUES (
                  'legacy-item', 'text', 'legacy-hash', 0,
                  '2026-05-04T00:00:00Z',
                  '2026-05-04T00:00:00Z',
                  '2026-05-04T00:00:00Z'
                );
                ",
            )
            .expect("legacy schema should initialize");

        apply_sync_migration(&connection).expect("sync migration should apply");
        let device = sync_service::initialize_device(&connection)
            .expect("device identity should initialize");

        assert!(!device.id.is_empty());
        assert!(has_column(&connection, "clipboard_items", "sync_id"));
        assert!(has_column(&connection, "clipboard_items", "deleted_at"));

        let stored_device_id: String = connection
            .query_row(
                "SELECT value FROM sync_state WHERE key = 'device_id'",
                [],
                |row| row.get(0),
            )
            .expect("device id should be stored");
        assert_eq!(stored_device_id, device.id);

        let item_device_id: String = connection
            .query_row(
                "SELECT device_id FROM clipboard_items WHERE id = 'legacy-item'",
                [],
                |row| row.get(0),
            )
            .expect("legacy item should be backfilled");
        assert_eq!(item_device_id, device.id);

        let device_count: i64 = connection
            .query_row(
                "SELECT COUNT(*) FROM devices WHERE id = ?1",
                params![device.id],
                |row| row.get(0),
            )
            .expect("device row should exist");
        assert_eq!(device_count, 1);
    }

    #[test]
    fn sync_blobs_migration_creates_image_blob_table() {
        let connection = Connection::open_in_memory().expect("in-memory sqlite should open");
        connection
            .execute_batch(
                "
                CREATE TABLE clipboard_items (
                  id TEXT PRIMARY KEY,
                  type TEXT NOT NULL,
                  hash TEXT NOT NULL,
                  copied_at TEXT NOT NULL,
                  created_at TEXT NOT NULL,
                  updated_at TEXT NOT NULL
                );
                ",
            )
            .expect("base schema should initialize");

        apply_sync_blobs_migration(&connection).expect("sync blob migration should apply");

        assert!(has_column(&connection, "sync_blobs", "blob_type"));
        assert!(has_column(&connection, "sync_blobs", "local_path"));
        assert!(has_column(&connection, "sync_blobs", "uploaded_at"));
    }

    #[test]
    fn sync_queue_cleanup_migration_adds_event_abandonment_column() {
        let connection = Connection::open_in_memory().expect("in-memory sqlite should open");
        connection
            .execute_batch(
                "
                CREATE TABLE clipboard_items (
                  id TEXT PRIMARY KEY,
                  type TEXT NOT NULL,
                  hash TEXT NOT NULL,
                  is_deleted INTEGER DEFAULT 0,
                  copied_at TEXT NOT NULL,
                  created_at TEXT NOT NULL,
                  updated_at TEXT NOT NULL,
                  sync_status TEXT DEFAULT 'pending',
                  deleted_at TEXT NULL
                );
                CREATE TABLE sync_events (
                  id TEXT PRIMARY KEY,
                  item_id TEXT,
                  event_type TEXT NOT NULL,
                  payload_json TEXT,
                  created_at TEXT NOT NULL,
                  synced_at TEXT NULL
                );
                CREATE TABLE sync_blobs (
                  id TEXT PRIMARY KEY,
                  item_id TEXT NOT NULL,
                  sync_status TEXT DEFAULT 'pending',
                  deleted_at TEXT NULL
                );
                ",
            )
            .expect("legacy sync schema should initialize");

        apply_sync_queue_cleanup_migration(&connection)
            .expect("sync queue cleanup migration should apply");
        apply_sync_queue_cleanup_migration(&connection)
            .expect("sync queue cleanup migration should be idempotent");

        assert!(has_column(&connection, "sync_events", "abandoned_at"));
    }

    #[test]
    fn database_optimize_runs_at_most_once_per_week() {
        let connection = Connection::open_in_memory().expect("in-memory sqlite should open");
        connection
            .execute_batch(
                "CREATE TABLE sync_state (
                   key TEXT PRIMARY KEY,
                   value TEXT NOT NULL,
                   updated_at TEXT NOT NULL
                 );",
            )
            .expect("sync state should initialize");

        assert!(optimize_database_if_due(&connection).expect("first optimize should run"));
        assert!(!optimize_database_if_due(&connection).expect("recent optimize should skip"));

        connection
            .execute(
                "UPDATE sync_state
                 SET value = datetime('now', '-8 days')
                 WHERE key = 'database_last_optimized_at'",
                [],
            )
            .expect("optimize timestamp should age");
        assert!(optimize_database_if_due(&connection).expect("old optimize should run again"));
    }

    #[test]
    fn startup_hides_legacy_privacy_placeholder_items() {
        let connection = Connection::open_in_memory().expect("in-memory sqlite should open");
        connection
            .execute_batch(
                "
                CREATE TABLE clipboard_items (
                  id TEXT PRIMARY KEY,
                  type TEXT NOT NULL,
                  title TEXT,
                  preview_text TEXT,
                  source_app TEXT,
                  is_deleted INTEGER DEFAULT 0,
                  deleted_at TEXT NULL,
                  sync_status TEXT DEFAULT 'synced'
                );

                CREATE TABLE clipboard_items_fts (
                  item_id TEXT PRIMARY KEY
                );

                INSERT INTO clipboard_items (
                  id, type, title, preview_text, source_app
                ) VALUES
                  ('legacy-private', 'text', '已隐藏敏感内容', '已隐藏疑似验证码等敏感内容', 'cliply'),
                  ('normal', 'text', 'hello', 'hello', 'cliply');

                INSERT INTO clipboard_items_fts (item_id) VALUES ('legacy-private'), ('normal');
                ",
            )
            .expect("schema should initialize");

        let changed = hide_legacy_privacy_placeholder_items(&connection)
            .expect("legacy privacy cleanup should run");

        assert_eq!(changed, 1);
        assert_eq!(is_deleted(&connection, "legacy-private"), 1);
        assert_eq!(is_deleted(&connection, "normal"), 0);
        assert_eq!(fts_count(&connection, "legacy-private"), 0);
    }

    fn has_column(connection: &Connection, table_name: &str, column_name: &str) -> bool {
        connection
            .prepare(&format!("PRAGMA table_info({table_name})"))
            .expect("pragma should prepare")
            .query_map([], |row| row.get::<_, String>(1))
            .expect("pragma should query")
            .collect::<Result<Vec<_>, _>>()
            .expect("pragma rows should collect")
            .iter()
            .any(|column| column == column_name)
    }

    fn is_deleted(connection: &Connection, item_id: &str) -> i64 {
        connection
            .query_row(
                "SELECT is_deleted FROM clipboard_items WHERE id = ?1",
                params![item_id],
                |row| row.get(0),
            )
            .expect("item should exist")
    }

    fn fts_count(connection: &Connection, item_id: &str) -> i64 {
        connection
            .query_row(
                "SELECT COUNT(*) FROM clipboard_items_fts WHERE item_id = ?1",
                params![item_id],
                |row| row.get(0),
            )
            .expect("fts count should load")
    }
}
