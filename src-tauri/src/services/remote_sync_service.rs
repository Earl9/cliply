use crate::error::CliplyError;
use crate::services::{
    database_service, sync_package_service, sync_service,
    sync_storage_provider::{LocalFolderSyncProvider, SyncProviderConfig, SyncStorageProvider},
};
use rusqlite::{params, Connection, OptionalExtension};
use serde::{Deserialize, Serialize};
use tauri::AppHandle;
use time::OffsetDateTime;

const CONFIG_KEY: &str = "remote_sync_provider_config";
const LAST_SYNCED_AT_KEY: &str = "remote_sync_last_synced_at";
const LAST_STATUS_KEY: &str = "remote_sync_last_status";
const LAST_ERROR_KEY: &str = "remote_sync_last_error";
const MANIFEST_PATH: &str = "CliplySync/manifest.json";
const SNAPSHOTS_PATH: &str = "CliplySync/snapshots";
const EVENTS_PATH: &str = "CliplySync/events";
const DEVICES_PATH: &str = "CliplySync/devices";

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RemoteSyncStatus {
    pub provider: SyncProviderConfig,
    pub manifest_exists: bool,
    pub last_synced_at: Option<String>,
    pub last_status: Option<String>,
    pub last_error: Option<String>,
    pub snapshot_count: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RemoteSyncResult {
    pub exported_count: usize,
    pub imported_count: usize,
    pub updated_count: usize,
    pub skipped_count: usize,
    pub deleted_count: usize,
    pub conflicted_count: usize,
    pub snapshot_count: usize,
    pub synced_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct RemoteSyncManifest {
    version: u32,
    app: String,
    created_at: String,
    updated_at: String,
    snapshots_path: String,
    events_path: String,
    devices_path: String,
}

pub fn get_remote_sync_status(app: &AppHandle) -> Result<RemoteSyncStatus, CliplyError> {
    let connection = database_service::connect(app)?;
    let provider = get_provider_config_from_connection(&connection)?;
    let mut manifest_exists = false;
    let mut snapshot_count = 0;

    if let SyncProviderConfig::LocalFolder { path } = &provider {
        let provider = LocalFolderSyncProvider::new(path);
        manifest_exists = provider.exists(MANIFEST_PATH)?;
        snapshot_count = list_snapshot_paths(&provider)?.len();
    }

    Ok(RemoteSyncStatus {
        provider,
        manifest_exists,
        last_synced_at: get_sync_state_value(&connection, LAST_SYNCED_AT_KEY)?,
        last_status: get_sync_state_value(&connection, LAST_STATUS_KEY)?,
        last_error: get_sync_state_value(&connection, LAST_ERROR_KEY)?,
        snapshot_count,
    })
}

pub fn set_remote_sync_provider(
    app: &AppHandle,
    config: SyncProviderConfig,
) -> Result<RemoteSyncStatus, CliplyError> {
    if !matches!(
        config,
        SyncProviderConfig::Disabled | SyncProviderConfig::LocalFolder { .. }
    ) {
        return Err(CliplyError::Sync(
            "当前版本只支持本地同步文件夹，其他 provider 暂未实现".to_string(),
        ));
    }

    let connection = database_service::connect(app)?;
    set_json_state_value(&connection, CONFIG_KEY, &config)?;
    if let SyncProviderConfig::LocalFolder { path } = &config {
        let provider = LocalFolderSyncProvider::new(path);
        ensure_local_folder_layout(&provider)?;
    }

    get_remote_sync_status(app)
}

pub fn export_to_remote_sync_folder(
    app: &AppHandle,
    password: String,
) -> Result<RemoteSyncResult, CliplyError> {
    let connection = database_service::connect(app)?;
    let provider = local_provider_from_connection(&connection)?;
    ensure_local_folder_layout(&provider)?;

    let device_id = sync_service::current_device_id(&connection)?;
    let (package_bytes, exported_at) =
        sync_package_service::build_sync_package_bytes(app, &password)?;
    let file_name = snapshot_file_name(&device_id, &exported_at);
    let snapshot_path = format!("{SNAPSHOTS_PATH}/{file_name}");
    provider.write(&snapshot_path, &package_bytes)?;
    write_device_marker(&provider, &device_id, &exported_at)?;
    write_manifest(&provider, &exported_at)?;

    let snapshot_count = list_snapshot_paths(&provider)?.len();
    set_remote_success(&connection, &exported_at)?;
    Ok(RemoteSyncResult {
        exported_count: 1,
        imported_count: 0,
        updated_count: 0,
        skipped_count: 0,
        deleted_count: 0,
        conflicted_count: 0,
        snapshot_count,
        synced_at: exported_at,
    })
}

pub fn import_from_remote_sync_folder(
    app: &AppHandle,
    password: String,
) -> Result<RemoteSyncResult, CliplyError> {
    let connection = database_service::connect(app)?;
    let provider = local_provider_from_connection(&connection)?;
    ensure_local_folder_layout(&provider)?;

    if !provider.exists(MANIFEST_PATH)? {
        return Err(CliplyError::Sync("同步目录缺少 manifest.json".to_string()));
    }

    let snapshot_paths = list_snapshot_paths(&provider)?;
    let mut imported_count = 0;
    let mut updated_count = 0;
    let mut skipped_count = 0;
    let mut deleted_count = 0;
    let mut conflicted_count = 0;

    for snapshot_path in &snapshot_paths {
        let bytes = provider.read(snapshot_path)?;
        let result = sync_package_service::import_sync_package_bytes(app, &bytes, &password)?;
        imported_count += result.imported_count;
        updated_count += result.updated_count;
        skipped_count += result.skipped_count;
        deleted_count += result.deleted_count;
        conflicted_count += result.conflicted_count;
    }

    let synced_at = current_timestamp()?;
    set_remote_success(&connection, &synced_at)?;
    Ok(RemoteSyncResult {
        exported_count: 0,
        imported_count,
        updated_count,
        skipped_count,
        deleted_count,
        conflicted_count,
        snapshot_count: snapshot_paths.len(),
        synced_at,
    })
}

fn ensure_local_folder_layout(provider: &LocalFolderSyncProvider) -> Result<(), CliplyError> {
    provider.write(&format!("{SNAPSHOTS_PATH}/.keep"), b"")?;
    provider.write(&format!("{EVENTS_PATH}/.keep"), b"")?;
    provider.write(&format!("{DEVICES_PATH}/.keep"), b"")?;
    provider.delete(&format!("{SNAPSHOTS_PATH}/.tmp"))?;
    if !provider.exists(MANIFEST_PATH)? {
        let now = current_timestamp()?;
        write_manifest(provider, &now)?;
    }
    Ok(())
}

fn write_manifest(provider: &LocalFolderSyncProvider, updated_at: &str) -> Result<(), CliplyError> {
    let created_at = if provider.exists(MANIFEST_PATH)? {
        read_manifest(provider)?
            .map(|manifest| manifest.created_at)
            .unwrap_or_else(|| updated_at.to_string())
    } else {
        updated_at.to_string()
    };
    let manifest = RemoteSyncManifest {
        version: 1,
        app: "Cliply".to_string(),
        created_at,
        updated_at: updated_at.to_string(),
        snapshots_path: "snapshots/".to_string(),
        events_path: "events/".to_string(),
        devices_path: "devices/".to_string(),
    };
    let bytes = serde_json::to_vec_pretty(&manifest)
        .map_err(|error| CliplyError::Sync(format!("同步 manifest 序列化失败: {error}")))?;
    provider.write(MANIFEST_PATH, &bytes)?;
    Ok(())
}

fn read_manifest(
    provider: &LocalFolderSyncProvider,
) -> Result<Option<RemoteSyncManifest>, CliplyError> {
    if !provider.exists(MANIFEST_PATH)? {
        return Ok(None);
    }
    let bytes = provider.read(MANIFEST_PATH)?;
    let manifest = serde_json::from_slice(&bytes)
        .map_err(|_| CliplyError::Sync("同步 manifest 格式不正确".to_string()))?;
    Ok(Some(manifest))
}

fn write_device_marker(
    provider: &LocalFolderSyncProvider,
    device_id: &str,
    exported_at: &str,
) -> Result<(), CliplyError> {
    let payload = serde_json::json!({
        "deviceId": device_id,
        "lastExportedAt": exported_at
    });
    let bytes = serde_json::to_vec_pretty(&payload)
        .map_err(|error| CliplyError::Sync(format!("设备同步状态序列化失败: {error}")))?;
    provider.write(&format!("{DEVICES_PATH}/{device_id}.json"), &bytes)
}

fn list_snapshot_paths(provider: &LocalFolderSyncProvider) -> Result<Vec<String>, CliplyError> {
    let mut paths = provider
        .list(SNAPSHOTS_PATH)?
        .into_iter()
        .filter(|entry| !entry.is_dir && entry.name.ends_with(".cliply-sync"))
        .map(|entry| entry.path)
        .collect::<Vec<_>>();
    paths.sort();
    Ok(paths)
}

fn local_provider_from_connection(
    connection: &Connection,
) -> Result<LocalFolderSyncProvider, CliplyError> {
    match get_provider_config_from_connection(connection)? {
        SyncProviderConfig::LocalFolder { path } if !path.trim().is_empty() => {
            Ok(LocalFolderSyncProvider::new(path))
        }
        SyncProviderConfig::Disabled => Err(CliplyError::Sync("远程同步已关闭".to_string())),
        _ => Err(CliplyError::Sync(
            "当前版本只支持本地同步文件夹 provider".to_string(),
        )),
    }
}

fn get_provider_config_from_connection(
    connection: &Connection,
) -> Result<SyncProviderConfig, CliplyError> {
    let raw = get_sync_state_value(connection, CONFIG_KEY)?;
    match raw {
        Some(value) => serde_json::from_str(&value)
            .map_err(|error| CliplyError::Sync(format!("同步 provider 配置损坏: {error}"))),
        None => Ok(SyncProviderConfig::Disabled),
    }
}

fn set_json_state_value<T: Serialize>(
    connection: &Connection,
    key: &str,
    value: &T,
) -> Result<(), CliplyError> {
    let value = serde_json::to_string(value)
        .map_err(|error| CliplyError::Sync(format!("同步配置序列化失败: {error}")))?;
    set_sync_state_value(connection, key, &value)
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

fn set_remote_success(connection: &Connection, timestamp: &str) -> Result<(), CliplyError> {
    set_sync_state_value(connection, LAST_SYNCED_AT_KEY, timestamp)?;
    set_sync_state_value(connection, LAST_STATUS_KEY, "success")?;
    set_sync_state_value(connection, LAST_ERROR_KEY, "")?;
    Ok(())
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

fn snapshot_file_name(device_id: &str, exported_at: &str) -> String {
    let safe_time = exported_at
        .chars()
        .map(|character| {
            if character.is_ascii_alphanumeric() {
                character
            } else {
                '-'
            }
        })
        .collect::<String>();
    format!("{safe_time}-{device_id}.cliply-sync")
}

fn current_timestamp() -> Result<String, CliplyError> {
    OffsetDateTime::now_utc()
        .format(&time::format_description::well_known::Rfc3339)
        .map_err(|error| CliplyError::StorageUnavailable(error.to_string()))
}

#[cfg(test)]
mod tests {
    use super::{
        ensure_local_folder_layout, list_snapshot_paths, snapshot_file_name,
        LocalFolderSyncProvider, MANIFEST_PATH, SNAPSHOTS_PATH,
    };
    use crate::services::sync_storage_provider::SyncStorageProvider;
    use std::fs;

    #[test]
    fn local_folder_layout_creates_manifest_and_directories() {
        let root =
            std::env::temp_dir().join(format!("cliply-remote-sync-test-{}", uuid::Uuid::new_v4()));
        let provider = LocalFolderSyncProvider::new(&root);

        ensure_local_folder_layout(&provider).expect("layout should initialize");

        assert!(provider.exists(MANIFEST_PATH).expect("manifest check"));
        assert!(provider.exists(SNAPSHOTS_PATH).expect("snapshots check"));

        fs::remove_dir_all(root).ok();
    }

    #[test]
    fn snapshot_listing_only_returns_sync_packages() {
        let root =
            std::env::temp_dir().join(format!("cliply-remote-sync-test-{}", uuid::Uuid::new_v4()));
        let provider = LocalFolderSyncProvider::new(&root);
        ensure_local_folder_layout(&provider).expect("layout should initialize");
        provider
            .write("CliplySync/snapshots/a.cliply-sync", b"one")
            .expect("snapshot should write");
        provider
            .write("CliplySync/snapshots/a.txt", b"ignored")
            .expect("extra file should write");

        let snapshots = list_snapshot_paths(&provider).expect("snapshots should list");

        assert_eq!(snapshots, vec!["CliplySync/snapshots/a.cliply-sync"]);
        fs::remove_dir_all(root).ok();
    }

    #[test]
    fn snapshot_names_are_filesystem_safe() {
        let name = snapshot_file_name("device-1", "2026-05-06T12:34:56+08:00");
        assert!(name.ends_with("-device-1.cliply-sync"));
        assert!(!name.contains(':'));
    }
}
