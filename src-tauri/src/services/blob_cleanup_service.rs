use crate::error::CliplyError;
use crate::logger;
use crate::services::database_service;
use rusqlite::{params, Connection};
use std::collections::HashSet;
use std::path::Path;
use std::thread;
use std::time::Duration;
use tauri::{AppHandle, Manager};

// Soft-deleted items keep their rows for sync tombstones, but their image
// files were never removed from disk. This background pass reclaims them and
// runs once per launch, delayed so it never competes with startup.
const CLEANUP_START_DELAY: Duration = Duration::from_secs(90);

#[derive(Debug, Default, Clone, Copy)]
pub struct BlobCleanupResult {
    pub removed_files: usize,
    pub freed_bytes: u64,
}

pub fn start_blob_cleanup_task(app: AppHandle) {
    let result = thread::Builder::new()
        .name("cliply-blob-cleanup".to_string())
        .spawn(move || {
            thread::sleep(CLEANUP_START_DELAY);
            match run_blob_cleanup(&app) {
                Ok(result) if result.removed_files > 0 => logger::info(
                    &app,
                    "blob_cleanup",
                    format!(
                        "removed_files={} freed_bytes={}",
                        result.removed_files, result.freed_bytes
                    ),
                ),
                Ok(_) => {}
                Err(error) => logger::error(&app, "blob_cleanup_failed", error),
            }
        });

    if result.is_err() {
        // Thread spawn failure is non-fatal; cleanup just runs next launch.
    }
}

pub fn run_blob_cleanup(app: &AppHandle) -> Result<BlobCleanupResult, CliplyError> {
    let connection = database_service::connect(app)?;
    let mut result = BlobCleanupResult::default();

    remove_deleted_item_files(&connection, &mut result)?;
    remove_orphan_blob_dir_files(app, &connection, &mut result)?;

    Ok(result)
}

/// Deletes files referenced by soft-deleted items (clipboard_formats.data_path
/// and sync_blobs.local_path), then clears the path columns so the rows keep
/// serving as sync tombstones without pointing at dead files.
fn remove_deleted_item_files(
    connection: &Connection,
    result: &mut BlobCleanupResult,
) -> Result<(), CliplyError> {
    let mut statement = connection.prepare(
        "SELECT cf.id, cf.data_path
         FROM clipboard_formats cf
         JOIN clipboard_items ci ON ci.id = cf.item_id
         WHERE ci.is_deleted = 1
           AND COALESCE(cf.data_path, '') <> ''",
    )?;
    let rows = statement.query_map([], |row| {
        Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
    })?;
    let mut format_rows = Vec::new();
    for row in rows {
        format_rows.push(row?);
    }
    drop(statement);

    for (format_id, path) in format_rows {
        remove_file_tracked(&path, result);
        connection.execute(
            "UPDATE clipboard_formats SET data_path = NULL WHERE id = ?1",
            params![format_id],
        )?;
    }

    let blob_rows = match collect_deleted_sync_blob_paths(connection) {
        Ok(rows) => rows,
        Err(error) if is_missing_sync_blobs_table(&error) => return Ok(()),
        Err(error) => return Err(error),
    };
    for (blob_id, path) in blob_rows {
        remove_file_tracked(&path, result);
        connection.execute(
            "UPDATE sync_blobs SET local_path = NULL WHERE id = ?1",
            params![blob_id],
        )?;
    }

    Ok(())
}

fn collect_deleted_sync_blob_paths(
    connection: &Connection,
) -> Result<Vec<(String, String)>, CliplyError> {
    let mut statement = connection.prepare(
        "SELECT sb.id, sb.local_path
         FROM sync_blobs sb
         JOIN clipboard_items ci ON ci.id = sb.item_id
         WHERE ci.is_deleted = 1
           AND COALESCE(sb.local_path, '') <> ''",
    )?;
    let rows = statement.query_map([], |row| {
        Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
    })?;
    let mut blob_rows = Vec::new();
    for row in rows {
        blob_rows.push(row?);
    }
    Ok(blob_rows)
}

/// Removes files in the blob directories that no live database row references
/// at all (e.g. rows hard-lost to a restored backup). File names start with
/// the item id, so membership is checked against live item ids.
fn remove_orphan_blob_dir_files(
    app: &AppHandle,
    connection: &Connection,
    result: &mut BlobCleanupResult,
) -> Result<(), CliplyError> {
    let mut live_ids = HashSet::new();
    let mut statement =
        connection.prepare("SELECT id FROM clipboard_items WHERE is_deleted = 0")?;
    let rows = statement.query_map([], |row| row.get::<_, String>(0))?;
    for row in rows {
        live_ids.insert(row?);
    }
    drop(statement);

    let app_data_dir = app
        .path()
        .app_data_dir()
        .map_err(|error| CliplyError::StorageUnavailable(error.to_string()))?;

    for child in ["images", "thumbnails", "sync-images"] {
        let dir = app_data_dir.join("blobs").join(child);
        let Ok(entries) = std::fs::read_dir(&dir) else {
            continue;
        };

        for entry in entries.flatten() {
            let path = entry.path();
            if !path.is_file() {
                continue;
            }
            let Some(stem) = path.file_stem().and_then(|value| value.to_str()) else {
                continue;
            };
            // sync-images files are "{id}-original" / "{id}-preview" etc.;
            // images and thumbnails are plain "{id}".
            let item_id = stem
                .strip_suffix("-original")
                .or_else(|| stem.strip_suffix("-compressed"))
                .or_else(|| stem.strip_suffix("-preview"))
                .unwrap_or(stem);
            if !live_ids.contains(item_id) && file_is_old_enough(&path) {
                remove_file_tracked(&path.to_string_lossy(), result);
            }
        }
    }

    Ok(())
}

/// A file written moments ago may belong to an ingest whose DB row has not
/// committed yet; only files older than an hour are treated as orphans.
fn file_is_old_enough(path: &Path) -> bool {
    std::fs::metadata(path)
        .and_then(|meta| meta.modified())
        .ok()
        .and_then(|modified| modified.elapsed().ok())
        .map(|age| age > Duration::from_secs(3600))
        .unwrap_or(false)
}

fn remove_file_tracked(path: &str, result: &mut BlobCleanupResult) {
    let path = Path::new(path);
    let size = std::fs::metadata(path).map(|meta| meta.len()).unwrap_or(0);
    if std::fs::remove_file(path).is_ok() {
        result.removed_files += 1;
        result.freed_bytes += size;
    }
}

fn is_missing_sync_blobs_table(error: &CliplyError) -> bool {
    error.to_string().contains("no such table: sync_blobs")
}
