use crate::error::CliplyError;
use crate::services::sync_service;
use crate::{db, logger};
use rusqlite::{params, Connection};
use tauri::AppHandle;

const INIT_MIGRATION: &str = include_str!("../db/migrations/001_init.sql");
const FTS_MIGRATION: &str = include_str!("../db/migrations/002_fts.sql");
const SYNC_MIGRATION: &str = include_str!("../db/migrations/003_sync.sql");

pub fn initialize(app: &AppHandle) -> Result<(), CliplyError> {
    let connection = connect(app)?;
    connection.execute_batch(INIT_MIGRATION)?;
    connection.execute_batch(FTS_MIGRATION)?;
    apply_sync_migration(&connection)?;
    let device = sync_service::initialize_device(&connection)?;
    logger::info(
        app,
        "sync_device_initialized",
        format!("device_id={}", device.id),
    );
    seed_mock_data(&connection)?;
    Ok(())
}

pub fn connect(app: &AppHandle) -> Result<Connection, CliplyError> {
    let path = db::database_path(app)?;
    let connection = Connection::open(path)?;
    connection.pragma_update(None, "foreign_keys", "ON")?;
    Ok(connection)
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
    use super::{apply_sync_migration, sync_service};
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
}
