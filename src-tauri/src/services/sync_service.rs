use crate::error::CliplyError;
use rusqlite::{params, Connection, OptionalExtension};
use serde_json::json;
use time::OffsetDateTime;
use uuid::Uuid;

const DEVICE_ID_KEY: &str = "device_id";
const SYNC_STATUS_PENDING: &str = "pending";

#[derive(Debug, Clone)]
pub struct LocalDevice {
    pub id: String,
}

pub fn initialize_device(connection: &Connection) -> Result<LocalDevice, CliplyError> {
    let now = current_timestamp()?;
    let device_id = connection
        .query_row(
            "SELECT value FROM sync_state WHERE key = ?1",
            params![DEVICE_ID_KEY],
            |row| row.get::<_, String>(0),
        )
        .optional()?
        .unwrap_or_else(|| format!("device-{}", Uuid::new_v4()));

    connection.execute(
        "INSERT INTO sync_state (key, value, updated_at)
         VALUES (?1, ?2, ?3)
         ON CONFLICT(key) DO UPDATE SET
           value = excluded.value,
           updated_at = excluded.updated_at",
        params![DEVICE_ID_KEY, device_id, now],
    )?;

    connection.execute(
        "INSERT INTO devices (id, name, platform, created_at, last_seen_at)
         VALUES (?1, ?2, ?3, ?4, ?4)
         ON CONFLICT(id) DO UPDATE SET
           name = excluded.name,
           platform = excluded.platform,
           last_seen_at = excluded.last_seen_at",
        params![device_id, current_device_name(), current_platform(), now],
    )?;

    backfill_clipboard_sync_fields(connection, &device_id)?;

    Ok(LocalDevice { id: device_id })
}

pub fn current_device_id(connection: &Connection) -> Result<String, CliplyError> {
    if let Some(device_id) = connection
        .query_row(
            "SELECT value FROM sync_state WHERE key = ?1",
            params![DEVICE_ID_KEY],
            |row| row.get::<_, String>(0),
        )
        .optional()?
    {
        return Ok(device_id);
    }

    Ok(initialize_device(connection)?.id)
}

pub fn mark_item_created(
    connection: &Connection,
    item_id: &str,
    item_type: &str,
    sync_id: &str,
    device_id: &str,
    created_at: &str,
) -> Result<(), CliplyError> {
    insert_sync_event(
        connection,
        item_id,
        "item_created",
        json!({
            "itemId": item_id,
            "syncId": sync_id,
            "deviceId": device_id,
            "type": item_type,
            "revision": 1
        }),
        created_at,
    )
}

pub fn mark_item_updated(
    connection: &Connection,
    item_id: &str,
    updated_at: &str,
) -> Result<(), CliplyError> {
    let (sync_id, device_id, revision, is_pinned) = connection.query_row(
        "SELECT COALESCE(sync_id, id),
                COALESCE(device_id, ''),
                COALESCE(revision, 1),
                is_pinned
         FROM clipboard_items
         WHERE id = ?1",
        params![item_id],
        |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, i64>(2)?,
                row.get::<_, i64>(3)?,
            ))
        },
    )?;

    insert_sync_event(
        connection,
        item_id,
        "item_updated",
        json!({
            "itemId": item_id,
            "syncId": sync_id,
            "deviceId": device_id,
            "revision": revision,
            "isPinned": is_pinned == 1
        }),
        updated_at,
    )
}

pub fn mark_item_deleted(
    connection: &Connection,
    item_id: &str,
    deleted_at: &str,
) -> Result<(), CliplyError> {
    let (sync_id, device_id, revision) = connection.query_row(
        "SELECT COALESCE(sync_id, id),
                COALESCE(device_id, ''),
                COALESCE(revision, 1)
         FROM clipboard_items
         WHERE id = ?1",
        params![item_id],
        |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, i64>(2)?,
            ))
        },
    )?;

    insert_sync_event(
        connection,
        item_id,
        "item_deleted",
        json!({
            "itemId": item_id,
            "syncId": sync_id,
            "deviceId": device_id,
            "revision": revision,
            "deletedAt": deleted_at
        }),
        deleted_at,
    )
}

fn backfill_clipboard_sync_fields(
    connection: &Connection,
    device_id: &str,
) -> Result<(), CliplyError> {
    connection.execute(
        "UPDATE clipboard_items
         SET sync_id = COALESCE(sync_id, id),
             device_id = COALESCE(device_id, ?1),
             revision = COALESCE(revision, 1),
             sync_status = COALESCE(sync_status, ?2),
             deleted_at = CASE
               WHEN deleted_at IS NULL AND is_deleted = 1 THEN updated_at
               ELSE deleted_at
             END
         WHERE sync_id IS NULL
            OR device_id IS NULL
            OR revision IS NULL
            OR sync_status IS NULL
            OR (deleted_at IS NULL AND is_deleted = 1)",
        params![device_id, SYNC_STATUS_PENDING],
    )?;

    Ok(())
}

fn insert_sync_event(
    connection: &Connection,
    item_id: &str,
    event_type: &str,
    payload: serde_json::Value,
    created_at: &str,
) -> Result<(), CliplyError> {
    connection.execute(
        "INSERT INTO sync_events (
            id, item_id, event_type, payload_json, created_at, synced_at
         ) VALUES (?1, ?2, ?3, ?4, ?5, NULL)",
        params![
            Uuid::new_v4().to_string(),
            item_id,
            event_type,
            payload.to_string(),
            created_at
        ],
    )?;

    Ok(())
}

fn current_device_name() -> String {
    std::env::var("COMPUTERNAME")
        .or_else(|_| std::env::var("HOSTNAME"))
        .map(|name| name.trim().to_string())
        .ok()
        .filter(|name| !name.is_empty())
        .unwrap_or_else(|| "Windows device".to_string())
}

fn current_platform() -> &'static str {
    std::env::consts::OS
}

fn current_timestamp() -> Result<String, CliplyError> {
    OffsetDateTime::now_utc()
        .format(&time::format_description::well_known::Rfc3339)
        .map_err(|error| CliplyError::StorageUnavailable(error.to_string()))
}
