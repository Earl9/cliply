use crate::error::CliplyError;
use crate::{platform, services::database_service};
use base64::engine::general_purpose::STANDARD as BASE64;
use base64::Engine;
use rusqlite::{params, OptionalExtension};
use serde::{Deserialize, Serialize};
use tauri::AppHandle;
use time::OffsetDateTime;

const SYNC_PASSWORD_KEY: &str = "remote_sync_password_protected";
const SYNC_PASSWORD_UPDATED_AT_KEY: &str = "remote_sync_password_updated_at";
const SECRET_PREFIX: &str = "dpapi-v1:";

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SyncPasswordStatus {
    pub saved: bool,
    pub updated_at: Option<String>,
}

pub fn save_sync_password(
    app: &AppHandle,
    password: &str,
) -> Result<SyncPasswordStatus, CliplyError> {
    if password.trim().is_empty() {
        return Err(CliplyError::Sync("同步密码不能为空".to_string()));
    }

    let connection = database_service::connect(app)?;
    let protected = platform::secure_storage::protect_bytes(password.as_bytes())?;
    let encoded = format!("{SECRET_PREFIX}{}", BASE64.encode(protected));
    let now = current_timestamp()?;
    set_sync_state_value(&connection, SYNC_PASSWORD_KEY, &encoded)?;
    set_sync_state_value(&connection, SYNC_PASSWORD_UPDATED_AT_KEY, &now)?;
    Ok(SyncPasswordStatus {
        saved: true,
        updated_at: Some(now),
    })
}

pub fn clear_sync_password(app: &AppHandle) -> Result<SyncPasswordStatus, CliplyError> {
    let connection = database_service::connect(app)?;
    connection.execute(
        "DELETE FROM sync_state WHERE key IN (?1, ?2)",
        params![SYNC_PASSWORD_KEY, SYNC_PASSWORD_UPDATED_AT_KEY],
    )?;
    Ok(SyncPasswordStatus {
        saved: false,
        updated_at: None,
    })
}

pub fn get_sync_password_status(app: &AppHandle) -> Result<SyncPasswordStatus, CliplyError> {
    let connection = database_service::connect(app)?;
    let saved = get_sync_state_value(&connection, SYNC_PASSWORD_KEY)?.is_some();
    Ok(SyncPasswordStatus {
        saved,
        updated_at: get_sync_state_value(&connection, SYNC_PASSWORD_UPDATED_AT_KEY)?,
    })
}

pub fn load_sync_password(app: &AppHandle) -> Result<Option<String>, CliplyError> {
    let connection = database_service::connect(app)?;
    let Some(raw) = get_sync_state_value(&connection, SYNC_PASSWORD_KEY)? else {
        return Ok(None);
    };
    let encoded = raw.strip_prefix(SECRET_PREFIX).ok_or_else(|| {
        CliplyError::Sync("本机同步密码格式不正确，请重新保存同步密码".to_string())
    })?;
    let protected = BASE64
        .decode(encoded)
        .map_err(|_| CliplyError::Sync("本机同步密码格式不正确，请重新保存同步密码".to_string()))?;
    let bytes = platform::secure_storage::unprotect_bytes(&protected)?;
    let password = String::from_utf8(bytes)
        .map_err(|_| CliplyError::Sync("本机同步密码格式不正确，请重新保存同步密码".to_string()))?;
    Ok(Some(password))
}

fn get_sync_state_value(
    connection: &rusqlite::Connection,
    key: &str,
) -> Result<Option<String>, CliplyError> {
    Ok(connection
        .query_row(
            "SELECT value FROM sync_state WHERE key = ?1",
            params![key],
            |row| row.get(0),
        )
        .optional()?)
}

fn set_sync_state_value(
    connection: &rusqlite::Connection,
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
