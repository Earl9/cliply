use crate::error::CliplyError;
use crate::logger;
use crate::services::{
    database_service, sync_blob_service, sync_package_service, sync_secret_service, sync_service,
    sync_storage_provider::{
        FtpSyncProvider, LocalFolderSyncProvider, SyncProviderConfig, SyncStorageProvider,
        WebdavSyncProvider,
    },
};
use rusqlite::{params, Connection, OptionalExtension};
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;
use tauri::AppHandle;
use time::{format_description::well_known::Rfc3339, Duration, OffsetDateTime};

const CONFIG_KEY: &str = "remote_sync_provider_config";
const LOCAL_FOLDER_CONFIG_KEY: &str = "remote_sync_provider_config_local_folder";
const WEBDAV_CONFIG_KEY: &str = "remote_sync_provider_config_webdav";
const FTP_CONFIG_KEY: &str = "remote_sync_provider_config_ftp";
const LAST_SYNCED_AT_KEY: &str = "remote_sync_last_synced_at";
const LAST_STATUS_KEY: &str = "remote_sync_last_status";
const LAST_ERROR_KEY: &str = "remote_sync_last_error";
const LAST_AUTO_SYNC_AT_KEY: &str = "remote_sync_last_auto_sync_at";
const LAST_AUTO_ATTEMPT_AT_KEY: &str = "remote_sync_last_auto_attempt_at";
const MANIFEST_EXISTS_KEY: &str = "remote_sync_manifest_exists";
const SNAPSHOT_COUNT_KEY: &str = "remote_sync_snapshot_count";
const IMPORTED_SNAPSHOTS_KEY: &str = "remote_sync_imported_snapshots";
const AUTO_SYNC_ENABLED_KEY: &str = "remote_sync_auto_enabled";
const AUTO_SYNC_INTERVAL_MINUTES_KEY: &str = "remote_sync_auto_interval_minutes";
const DEFAULT_AUTO_SYNC_INTERVAL_MINUTES: u64 = 5;
const MIN_AUTO_SYNC_INTERVAL_MINUTES: u64 = 1;
const MAX_AUTO_SYNC_INTERVAL_MINUTES: u64 = 24 * 60;
const MANIFEST_PATH: &str = "CliplySync/manifest.json";
const SNAPSHOTS_PATH: &str = "CliplySync/snapshots";
const EVENTS_PATH: &str = "CliplySync/events";
const DEVICES_PATH: &str = "CliplySync/devices";
const BLOBS_PATH: &str = "CliplySync/blobs";

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RemoteSyncStatus {
    pub provider: SyncProviderConfig,
    pub saved_provider_configs: SavedSyncProviderConfigs,
    pub manifest_exists: bool,
    pub last_synced_at: Option<String>,
    pub last_status: Option<String>,
    pub last_error: Option<String>,
    pub snapshot_count: usize,
    pub auto_sync_enabled: bool,
    pub auto_sync_interval_minutes: u64,
    pub sync_password_saved: bool,
    pub sync_password_updated_at: Option<String>,
    pub last_auto_sync_at: Option<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SavedSyncProviderConfigs {
    pub local_folder: Option<SyncProviderConfig>,
    pub webdav: Option<SyncProviderConfig>,
    pub ftp: Option<SyncProviderConfig>,
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
    #[serde(default = "default_manifest_blobs_path")]
    blobs_path: String,
}

pub fn get_remote_sync_status(app: &AppHandle) -> Result<RemoteSyncStatus, CliplyError> {
    let connection = database_service::connect(app)?;
    let provider = get_provider_config_from_connection(&connection)?;
    let mut manifest_exists = get_bool_sync_state_value(&connection, MANIFEST_EXISTS_KEY)?;
    let mut snapshot_count = get_usize_sync_state_value(&connection, SNAPSHOT_COUNT_KEY)?;
    let mut current_error = None;

    if let SyncProviderConfig::LocalFolder { .. } = provider {
        match provider_from_config(&provider).and_then(|provider_impl| {
            let manifest_exists = provider_impl.exists(MANIFEST_PATH)?;
            let snapshot_count = list_snapshot_paths(provider_impl.as_ref())?.len();
            Ok((manifest_exists, snapshot_count))
        }) {
            Ok((exists, count)) => {
                manifest_exists = exists;
                snapshot_count = count;
            }
            Err(error) => {
                current_error = Some(error.to_string());
            }
        }
    }

    build_remote_sync_status(
        app,
        &connection,
        provider,
        manifest_exists,
        snapshot_count,
        current_error,
    )
}

pub fn set_remote_sync_provider(
    app: &AppHandle,
    config: SyncProviderConfig,
) -> Result<RemoteSyncStatus, CliplyError> {
    if !matches!(
        config,
        SyncProviderConfig::Disabled
            | SyncProviderConfig::LocalFolder { .. }
            | SyncProviderConfig::Webdav { .. }
            | SyncProviderConfig::Ftp { .. }
    ) {
        return Err(CliplyError::Sync(
            "当前版本只支持本地文件夹、WebDAV 和 FTP/FTPS provider".to_string(),
        ));
    }

    let connection = database_service::connect(app)?;
    let config = merge_saved_provider_password(&connection, config)?;
    validate_provider_config(&config)?;
    set_json_state_value(&connection, CONFIG_KEY, &config)?;
    cache_provider_config(&connection, &config)?;

    build_remote_sync_status(
        app,
        &connection,
        config,
        get_bool_sync_state_value(&connection, MANIFEST_EXISTS_KEY)?,
        get_usize_sync_state_value(&connection, SNAPSHOT_COUNT_KEY)?,
        None,
    )
}

pub fn update_auto_sync_config(
    app: &AppHandle,
    enabled: bool,
    interval_minutes: u64,
    password: Option<String>,
) -> Result<RemoteSyncStatus, CliplyError> {
    if let Some(password) = password.as_deref().filter(|value| !value.trim().is_empty()) {
        sync_secret_service::save_sync_password(app, password)?;
    }

    let password_status = sync_secret_service::get_sync_password_status(app)?;
    if enabled && !password_status.saved {
        return Err(CliplyError::Sync(
            "开启自动同步前需要先保存同步密码到本机".to_string(),
        ));
    }

    let connection = database_service::connect(app)?;
    let interval_minutes = normalize_auto_sync_interval(interval_minutes);
    set_sync_state_value(
        &connection,
        AUTO_SYNC_ENABLED_KEY,
        if enabled { "true" } else { "false" },
    )?;
    set_sync_state_value(
        &connection,
        AUTO_SYNC_INTERVAL_MINUTES_KEY,
        &interval_minutes.to_string(),
    )?;

    get_remote_sync_status(app)
}

pub fn clear_auto_sync_password(app: &AppHandle) -> Result<RemoteSyncStatus, CliplyError> {
    sync_secret_service::clear_sync_password(app)?;
    let connection = database_service::connect(app)?;
    set_sync_state_value(&connection, AUTO_SYNC_ENABLED_KEY, "false")?;
    get_remote_sync_status(app)
}

pub fn export_to_remote_sync_folder(
    app: &AppHandle,
    password: String,
) -> Result<RemoteSyncResult, CliplyError> {
    export_remote_snapshot(app, &password)
}

pub fn import_from_remote_sync_folder(
    app: &AppHandle,
    password: String,
) -> Result<RemoteSyncResult, CliplyError> {
    import_remote_snapshots(app, &password, false)
}

pub fn sync_with_remote_now(
    app: &AppHandle,
    password: Option<String>,
) -> Result<RemoteSyncResult, CliplyError> {
    let password = resolve_sync_password(app, password)?;
    sync_with_remote(app, &password, false, false)
}

pub fn run_auto_sync_cycle(app: &AppHandle) -> Result<Option<RemoteSyncResult>, CliplyError> {
    let connection = database_service::connect(app)?;
    if !get_bool_sync_state_value(&connection, AUTO_SYNC_ENABLED_KEY)? {
        return Ok(None);
    }

    let provider = get_provider_config_from_connection(&connection)?;
    if matches!(provider, SyncProviderConfig::Disabled) {
        return Ok(None);
    }

    let interval_minutes = get_auto_sync_interval_minutes(&connection)?;
    if !auto_sync_is_due(&connection, interval_minutes)? {
        return Ok(None);
    }

    let Some(password) = sync_secret_service::load_sync_password(app)? else {
        set_sync_state_value(&connection, LAST_STATUS_KEY, "error")?;
        set_sync_state_value(&connection, LAST_ERROR_KEY, "自动同步缺少本机同步密码")?;
        return Ok(None);
    };

    let attempted_at = current_timestamp()?;
    set_sync_state_value(&connection, LAST_AUTO_ATTEMPT_AT_KEY, &attempted_at)?;
    set_sync_state_value(&connection, LAST_STATUS_KEY, "syncing")?;

    match sync_with_remote(app, &password, true, false) {
        Ok(result) => {
            let connection = database_service::connect(app)?;
            set_sync_state_value(&connection, LAST_AUTO_SYNC_AT_KEY, &result.synced_at)?;
            Ok(Some(result))
        }
        Err(error) => {
            record_remote_sync_error(app, &error)?;
            Err(error)
        }
    }
}

fn sync_with_remote(
    app: &AppHandle,
    password: &str,
    only_new_snapshots: bool,
    force_export: bool,
) -> Result<RemoteSyncResult, CliplyError> {
    let import_result = import_remote_snapshots(app, password, only_new_snapshots)?;
    let connection = database_service::connect(app)?;
    let pending_count = pending_sync_change_count(&connection)?;
    let should_export = force_export || pending_count > 0 || import_result.snapshot_count == 0;

    if !should_export {
        return Ok(import_result);
    }

    let export_result = export_remote_snapshot(app, password)?;
    Ok(RemoteSyncResult {
        exported_count: export_result.exported_count,
        imported_count: import_result.imported_count,
        updated_count: import_result.updated_count,
        skipped_count: import_result.skipped_count,
        deleted_count: import_result.deleted_count,
        conflicted_count: import_result.conflicted_count,
        snapshot_count: export_result.snapshot_count,
        synced_at: export_result.synced_at,
    })
}

fn export_remote_snapshot(
    app: &AppHandle,
    password: &str,
) -> Result<RemoteSyncResult, CliplyError> {
    let connection = database_service::connect(app)?;
    let provider_config = get_provider_config_from_connection(&connection)?;
    let provider = provider_from_config(&provider_config)?;
    ensure_remote_layout(provider.as_ref())?;

    let device_id = sync_service::current_device_id(&connection)?;
    let _ = upload_pending_image_blobs(app, provider.as_ref(), &connection, password)?;
    let (package_bytes, exported_at) =
        sync_package_service::build_sync_package_bytes(app, password)?;
    let file_name = snapshot_file_name(&device_id, &exported_at);
    let snapshot_path = format!("{SNAPSHOTS_PATH}/{file_name}");
    provider.write(&snapshot_path, &package_bytes)?;
    write_device_marker(provider.as_ref(), &device_id, &exported_at)?;
    write_manifest(provider.as_ref(), &exported_at)?;
    mark_sync_exported(&connection, &exported_at)?;
    record_imported_snapshot(&connection, &snapshot_path)?;

    let snapshot_count = list_snapshot_paths(provider.as_ref())?.len();
    set_remote_success(&connection, &exported_at, snapshot_count)?;
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

fn import_remote_snapshots(
    app: &AppHandle,
    password: &str,
    only_new_snapshots: bool,
) -> Result<RemoteSyncResult, CliplyError> {
    let connection = database_service::connect(app)?;
    let provider_config = get_provider_config_from_connection(&connection)?;
    let provider = provider_from_config(&provider_config)?;
    ensure_remote_layout(provider.as_ref())?;

    let snapshot_paths = list_snapshot_paths(provider.as_ref())?;
    let imported_snapshots = if only_new_snapshots {
        get_imported_snapshots(&connection)?
    } else {
        BTreeSet::new()
    };
    let mut imported_count = 0;
    let mut updated_count = 0;
    let mut skipped_count = 0;
    let mut deleted_count = 0;
    let mut conflicted_count = 0;

    for snapshot_path in &snapshot_paths {
        if imported_snapshots.contains(snapshot_path) {
            continue;
        }

        let bytes = provider.read(snapshot_path)?;
        let result = sync_package_service::import_sync_package_bytes(app, &bytes, password)?;
        record_imported_snapshot(&connection, snapshot_path)?;
        imported_count += result.imported_count;
        updated_count += result.updated_count;
        skipped_count += result.skipped_count;
        deleted_count += result.deleted_count;
        conflicted_count += result.conflicted_count;
        let _ = download_missing_image_blobs(app, provider.as_ref(), &connection, password)?;
    }

    let synced_at = current_timestamp()?;
    set_remote_success(&connection, &synced_at, snapshot_paths.len())?;
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

fn ensure_remote_layout(provider: &dyn SyncStorageProvider) -> Result<(), CliplyError> {
    ensure_keep_file(provider, &format!("{SNAPSHOTS_PATH}/.keep"))?;
    ensure_keep_file(provider, &format!("{EVENTS_PATH}/.keep"))?;
    ensure_keep_file(provider, &format!("{DEVICES_PATH}/.keep"))?;
    ensure_keep_file(provider, &format!("{BLOBS_PATH}/.keep"))?;
    remove_legacy_tmp_file_if_present(provider)?;
    if !provider.exists(MANIFEST_PATH)? {
        let now = current_timestamp()?;
        write_manifest(provider, &now)?;
    }
    Ok(())
}

fn ensure_keep_file(provider: &dyn SyncStorageProvider, path: &str) -> Result<(), CliplyError> {
    if !provider.exists(path)? {
        provider.write(path, b"")?;
    }
    Ok(())
}

fn remove_legacy_tmp_file_if_present(
    provider: &dyn SyncStorageProvider,
) -> Result<(), CliplyError> {
    let tmp_path = format!("{SNAPSHOTS_PATH}/.tmp");
    let exists = provider
        .list(SNAPSHOTS_PATH)?
        .iter()
        .any(|entry| entry.path == tmp_path || entry.name == ".tmp");
    if exists {
        provider.delete(&tmp_path)?;
    }
    Ok(())
}

fn write_manifest(provider: &dyn SyncStorageProvider, updated_at: &str) -> Result<(), CliplyError> {
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
        blobs_path: "blobs/".to_string(),
    };
    let bytes = serde_json::to_vec_pretty(&manifest)
        .map_err(|error| CliplyError::Sync(format!("同步 manifest 序列化失败: {error}")))?;
    provider.write(MANIFEST_PATH, &bytes)?;
    Ok(())
}

fn read_manifest(
    provider: &dyn SyncStorageProvider,
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
    provider: &dyn SyncStorageProvider,
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

fn upload_pending_image_blobs(
    app: &AppHandle,
    provider: &dyn SyncStorageProvider,
    connection: &Connection,
    password: &str,
) -> Result<usize, CliplyError> {
    let blobs = sync_blob_service::load_uploadable_blobs(connection)?;
    let uploaded_at = current_timestamp()?;
    let mut uploaded = 0;

    for blob in blobs {
        let remote_path = sync_blob_service::remote_blob_path(&blob);
        let bytes = sync_blob_service::build_remote_blob_envelope(&blob, password)?;
        if !provider.exists(&remote_path)? {
            provider.write(&remote_path, &bytes)?;
        }
        sync_blob_service::mark_blob_uploaded(connection, &blob.id, &remote_path, &uploaded_at)?;
        uploaded += 1;
    }

    if uploaded > 0 {
        logger::info(
            app,
            "image_sync_blobs_uploaded",
            format!("count={uploaded}"),
        );
    }

    Ok(uploaded)
}

fn download_missing_image_blobs(
    app: &AppHandle,
    provider: &dyn SyncStorageProvider,
    connection: &Connection,
    password: &str,
) -> Result<usize, CliplyError> {
    let blobs = sync_blob_service::load_missing_local_blobs(connection)?;
    let mut downloaded = 0;

    for blob in blobs {
        let Some(remote_path) = blob.remote_path.as_deref() else {
            continue;
        };
        if !provider.exists(remote_path)? {
            logger::error(
                app,
                "image_sync_blob_missing_remote",
                format!("blob_id={} remote_path={remote_path}", blob.id),
            );
            continue;
        }

        let bytes = provider.read(remote_path)?;
        let (envelope, plaintext) =
            sync_blob_service::decrypt_remote_blob_envelope(&bytes, password)?;
        sync_blob_service::write_downloaded_blob(app, connection, &blob, &envelope, &plaintext)?;
        downloaded += 1;
    }

    if downloaded > 0 {
        logger::info(
            app,
            "image_sync_blobs_downloaded",
            format!("count={downloaded}"),
        );
    }

    Ok(downloaded)
}

fn list_snapshot_paths(provider: &dyn SyncStorageProvider) -> Result<Vec<String>, CliplyError> {
    let mut paths = provider
        .list(SNAPSHOTS_PATH)?
        .into_iter()
        .filter(|entry| !entry.is_dir && entry.name.ends_with(".cliply-sync"))
        .map(|entry| entry.path)
        .collect::<Vec<_>>();
    paths.sort();
    Ok(paths)
}

fn build_remote_sync_status(
    app: &AppHandle,
    connection: &Connection,
    provider: SyncProviderConfig,
    manifest_exists: bool,
    snapshot_count: usize,
    current_error: Option<String>,
) -> Result<RemoteSyncStatus, CliplyError> {
    let password_status = sync_secret_service::get_sync_password_status(app)?;
    let last_status = if current_error.is_some() {
        Some("error".to_string())
    } else {
        get_sync_state_value(connection, LAST_STATUS_KEY)?
    };

    Ok(redact_remote_sync_status(RemoteSyncStatus {
        provider,
        saved_provider_configs: get_saved_provider_configs(connection)?,
        manifest_exists,
        last_synced_at: get_sync_state_value(connection, LAST_SYNCED_AT_KEY)?,
        last_status,
        last_error: current_error.or(get_sync_state_value(connection, LAST_ERROR_KEY)?),
        snapshot_count,
        auto_sync_enabled: get_bool_sync_state_value(connection, AUTO_SYNC_ENABLED_KEY)?,
        auto_sync_interval_minutes: get_auto_sync_interval_minutes(connection)?,
        sync_password_saved: password_status.saved,
        sync_password_updated_at: password_status.updated_at,
        last_auto_sync_at: get_sync_state_value(connection, LAST_AUTO_SYNC_AT_KEY)?,
    }))
}

fn redact_remote_sync_status(mut status: RemoteSyncStatus) -> RemoteSyncStatus {
    status.provider = redact_provider_password(status.provider);
    status.saved_provider_configs.local_folder = status
        .saved_provider_configs
        .local_folder
        .map(redact_provider_password);
    status.saved_provider_configs.webdav = status
        .saved_provider_configs
        .webdav
        .map(redact_provider_password);
    status.saved_provider_configs.ftp = status
        .saved_provider_configs
        .ftp
        .map(redact_provider_password);
    status
}

fn redact_provider_password(config: SyncProviderConfig) -> SyncProviderConfig {
    match config {
        SyncProviderConfig::Webdav {
            url,
            username,
            remote_path,
            ..
        } => SyncProviderConfig::Webdav {
            url,
            username,
            password: String::new(),
            remote_path,
        },
        SyncProviderConfig::Ftp {
            host,
            port,
            username,
            secure,
            remote_path,
            ..
        } => SyncProviderConfig::Ftp {
            host,
            port,
            username,
            password: String::new(),
            secure,
            remote_path,
        },
        config => config,
    }
}

fn resolve_sync_password(app: &AppHandle, password: Option<String>) -> Result<String, CliplyError> {
    if let Some(password) = password.filter(|value| !value.trim().is_empty()) {
        return Ok(password);
    }

    sync_secret_service::load_sync_password(app)?.ok_or_else(|| {
        CliplyError::Sync("请先输入同步密码，或将同步密码保存到本机用于自动同步".to_string())
    })
}

fn pending_sync_change_count(connection: &Connection) -> Result<i64, CliplyError> {
    let item_count: i64 = connection.query_row(
        "SELECT COUNT(*)
         FROM clipboard_items
         WHERE sync_status = 'pending'",
        [],
        |row| row.get(0),
    )?;
    let event_count: i64 = connection.query_row(
        "SELECT COUNT(*)
         FROM sync_events
         WHERE synced_at IS NULL",
        [],
        |row| row.get(0),
    )?;
    let blob_count = sync_blob_service::pending_blob_change_count(connection)?;
    Ok(item_count + event_count + blob_count)
}

fn mark_sync_exported(connection: &Connection, exported_at: &str) -> Result<(), CliplyError> {
    connection.execute(
        "UPDATE clipboard_items
         SET sync_status = 'synced',
             last_synced_at = ?1
         WHERE sync_status = 'pending'",
        params![exported_at],
    )?;
    connection.execute(
        "UPDATE sync_events
         SET synced_at = ?1
         WHERE synced_at IS NULL",
        params![exported_at],
    )?;
    sync_blob_service::mark_pending_blob_tombstones_exported(connection, exported_at)?;
    Ok(())
}

fn get_imported_snapshots(connection: &Connection) -> Result<BTreeSet<String>, CliplyError> {
    let Some(value) = get_sync_state_value(connection, IMPORTED_SNAPSHOTS_KEY)? else {
        return Ok(BTreeSet::new());
    };

    let snapshots = serde_json::from_str::<Vec<String>>(&value)
        .unwrap_or_default()
        .into_iter()
        .collect::<BTreeSet<_>>();
    Ok(snapshots)
}

fn record_imported_snapshot(
    connection: &Connection,
    snapshot_path: &str,
) -> Result<(), CliplyError> {
    let mut snapshots = get_imported_snapshots(connection)?;
    snapshots.insert(snapshot_path.to_string());
    let mut snapshots = snapshots.into_iter().collect::<Vec<_>>();
    if snapshots.len() > 1000 {
        snapshots = snapshots.split_off(snapshots.len() - 1000);
    }
    let value = serde_json::to_string(&snapshots)
        .map_err(|error| CliplyError::Sync(format!("同步快照状态序列化失败: {error}")))?;
    set_sync_state_value(connection, IMPORTED_SNAPSHOTS_KEY, &value)
}

fn get_auto_sync_interval_minutes(connection: &Connection) -> Result<u64, CliplyError> {
    Ok(
        get_sync_state_value(connection, AUTO_SYNC_INTERVAL_MINUTES_KEY)?
            .and_then(|value| value.parse::<u64>().ok())
            .map(normalize_auto_sync_interval)
            .unwrap_or(DEFAULT_AUTO_SYNC_INTERVAL_MINUTES),
    )
}

fn normalize_auto_sync_interval(value: u64) -> u64 {
    value.clamp(
        MIN_AUTO_SYNC_INTERVAL_MINUTES,
        MAX_AUTO_SYNC_INTERVAL_MINUTES,
    )
}

fn auto_sync_is_due(connection: &Connection, interval_minutes: u64) -> Result<bool, CliplyError> {
    let Some(last_attempt_at) = get_sync_state_value(connection, LAST_AUTO_ATTEMPT_AT_KEY)? else {
        return Ok(true);
    };
    let Ok(last_attempt_at) = OffsetDateTime::parse(&last_attempt_at, &Rfc3339) else {
        return Ok(true);
    };

    Ok(OffsetDateTime::now_utc() - last_attempt_at
        >= Duration::minutes(normalize_auto_sync_interval(interval_minutes) as i64))
}

fn record_remote_sync_error(app: &AppHandle, error: &CliplyError) -> Result<(), CliplyError> {
    let connection = database_service::connect(app)?;
    set_sync_state_value(&connection, LAST_STATUS_KEY, "error")?;
    set_sync_state_value(&connection, LAST_ERROR_KEY, &error.to_string())?;
    Ok(())
}

fn cache_provider_config(
    connection: &Connection,
    config: &SyncProviderConfig,
) -> Result<(), CliplyError> {
    let key = match config {
        SyncProviderConfig::LocalFolder { .. } => LOCAL_FOLDER_CONFIG_KEY,
        SyncProviderConfig::Webdav { .. } => WEBDAV_CONFIG_KEY,
        SyncProviderConfig::Ftp { .. } => FTP_CONFIG_KEY,
        SyncProviderConfig::Disabled
        | SyncProviderConfig::Sftp { .. }
        | SyncProviderConfig::S3 { .. } => return Ok(()),
    };
    set_json_state_value(connection, key, config)
}

fn get_saved_provider_configs(
    connection: &Connection,
) -> Result<SavedSyncProviderConfigs, CliplyError> {
    let active = get_provider_config_from_connection(connection)?;
    let mut configs = SavedSyncProviderConfigs {
        local_folder: get_cached_provider_config(
            connection,
            LOCAL_FOLDER_CONFIG_KEY,
            "local-folder",
        )?,
        webdav: get_cached_provider_config(connection, WEBDAV_CONFIG_KEY, "webdav")?,
        ftp: get_cached_provider_config(connection, FTP_CONFIG_KEY, "ftp")?,
    };

    match active {
        SyncProviderConfig::LocalFolder { .. } => configs.local_folder = Some(active),
        SyncProviderConfig::Webdav { .. } => configs.webdav = Some(active),
        SyncProviderConfig::Ftp { .. } => configs.ftp = Some(active),
        _ => {}
    }

    Ok(configs)
}

fn merge_saved_provider_password(
    connection: &Connection,
    config: SyncProviderConfig,
) -> Result<SyncProviderConfig, CliplyError> {
    match config {
        SyncProviderConfig::Webdav {
            url,
            username,
            password,
            remote_path,
        } if password.is_empty() => Ok(SyncProviderConfig::Webdav {
            url,
            username,
            password: saved_provider_password(connection, "webdav")?.unwrap_or_default(),
            remote_path,
        }),
        SyncProviderConfig::Ftp {
            host,
            port,
            username,
            password,
            secure,
            remote_path,
        } if password.is_empty() => Ok(SyncProviderConfig::Ftp {
            host,
            port,
            username,
            password: saved_provider_password(connection, "ftp")?.unwrap_or_default(),
            secure,
            remote_path,
        }),
        config => Ok(config),
    }
}

fn saved_provider_password(
    connection: &Connection,
    expected_type: &str,
) -> Result<Option<String>, CliplyError> {
    let active = get_provider_config_from_connection(connection)?;
    if provider_type_matches(&active, expected_type) {
        if let Some(password) = provider_password(&active).filter(|value| !value.is_empty()) {
            return Ok(Some(password.to_string()));
        }
    }

    let key = match expected_type {
        "webdav" => WEBDAV_CONFIG_KEY,
        "ftp" => FTP_CONFIG_KEY,
        _ => return Ok(None),
    };
    let cached = get_cached_provider_config(connection, key, expected_type)?;
    Ok(cached
        .as_ref()
        .and_then(provider_password)
        .filter(|value| !value.is_empty())
        .map(ToString::to_string))
}

fn provider_type_matches(config: &SyncProviderConfig, expected_type: &str) -> bool {
    matches!(
        (config, expected_type),
        (SyncProviderConfig::Webdav { .. }, "webdav") | (SyncProviderConfig::Ftp { .. }, "ftp")
    )
}

fn provider_password(config: &SyncProviderConfig) -> Option<&str> {
    match config {
        SyncProviderConfig::Webdav { password, .. } | SyncProviderConfig::Ftp { password, .. } => {
            Some(password)
        }
        _ => None,
    }
}

fn get_cached_provider_config(
    connection: &Connection,
    key: &str,
    expected_type: &str,
) -> Result<Option<SyncProviderConfig>, CliplyError> {
    let Some(value) = get_sync_state_value(connection, key)? else {
        return Ok(None);
    };

    let config: SyncProviderConfig = serde_json::from_str(&value)
        .map_err(|error| CliplyError::Sync(format!("同步 provider 缓存配置损坏: {error}")))?;
    let matches_type = matches!(
        (&config, expected_type),
        (SyncProviderConfig::LocalFolder { .. }, "local-folder")
            | (SyncProviderConfig::Webdav { .. }, "webdav")
            | (SyncProviderConfig::Ftp { .. }, "ftp")
    );
    Ok(matches_type.then_some(config))
}

fn provider_from_config(
    config: &SyncProviderConfig,
) -> Result<Box<dyn SyncStorageProvider>, CliplyError> {
    validate_provider_config(config)?;
    match config {
        SyncProviderConfig::LocalFolder { path } => {
            Ok(Box::new(LocalFolderSyncProvider::new(path)))
        }
        SyncProviderConfig::Webdav {
            url,
            username,
            password,
            remote_path,
        } => Ok(Box::new(WebdavSyncProvider::new(
            url.clone(),
            username.clone(),
            password.clone(),
            remote_path.clone(),
        ))),
        SyncProviderConfig::Ftp {
            host,
            port,
            username,
            password,
            secure,
            remote_path,
        } => Ok(Box::new(FtpSyncProvider::new(
            host.clone(),
            *port,
            username.clone(),
            password.clone(),
            *secure,
            remote_path.clone(),
        ))),
        _ => Err(CliplyError::Sync("当前 provider 尚未实现".to_string())),
    }
}

fn validate_provider_config(config: &SyncProviderConfig) -> Result<(), CliplyError> {
    match config {
        SyncProviderConfig::Disabled => Ok(()),
        SyncProviderConfig::LocalFolder { path } if !path.trim().is_empty() => Ok(()),
        SyncProviderConfig::LocalFolder { .. } => Err(CliplyError::Sync(
            "本地同步文件夹需要先选择目录".to_string(),
        )),
        SyncProviderConfig::Webdav {
            url,
            username,
            password,
            ..
        } if !url.trim().is_empty() && !username.trim().is_empty() && !password.is_empty() => {
            WebdavSyncProvider::validate_url(url)
        }
        SyncProviderConfig::Webdav { .. } => Err(CliplyError::Sync(
            "WebDAV 同步需要填写地址、用户名和密码".to_string(),
        )),
        SyncProviderConfig::Ftp {
            host,
            port,
            username,
            password,
            ..
        } if !host.trim().is_empty()
            && *port > 0
            && !username.trim().is_empty()
            && !password.is_empty() =>
        {
            Ok(())
        }
        SyncProviderConfig::Ftp { .. } => Err(CliplyError::Sync(
            "FTP 同步需要填写主机、端口、用户名和密码".to_string(),
        )),
        _ => Err(CliplyError::Sync("当前 provider 尚未实现".to_string())),
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

fn set_remote_success(
    connection: &Connection,
    timestamp: &str,
    snapshot_count: usize,
) -> Result<(), CliplyError> {
    set_sync_state_value(connection, LAST_SYNCED_AT_KEY, timestamp)?;
    set_sync_state_value(connection, LAST_STATUS_KEY, "success")?;
    set_sync_state_value(connection, LAST_ERROR_KEY, "")?;
    set_sync_state_value(connection, MANIFEST_EXISTS_KEY, "true")?;
    set_sync_state_value(connection, SNAPSHOT_COUNT_KEY, &snapshot_count.to_string())?;
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

fn default_manifest_blobs_path() -> String {
    "blobs/".to_string()
}

fn current_timestamp() -> Result<String, CliplyError> {
    OffsetDateTime::now_utc()
        .format(&time::format_description::well_known::Rfc3339)
        .map_err(|error| CliplyError::StorageUnavailable(error.to_string()))
}

#[cfg(test)]
mod tests {
    use super::{
        ensure_remote_layout, list_snapshot_paths, redact_provider_password,
        redact_remote_sync_status, snapshot_file_name, LocalFolderSyncProvider, RemoteSyncStatus,
        SavedSyncProviderConfigs, SyncProviderConfig, MANIFEST_PATH, SNAPSHOTS_PATH,
    };
    use crate::services::sync_storage_provider::SyncStorageProvider;
    use std::fs;

    #[test]
    fn local_folder_layout_creates_manifest_and_directories() {
        let root =
            std::env::temp_dir().join(format!("cliply-remote-sync-test-{}", uuid::Uuid::new_v4()));
        let provider = LocalFolderSyncProvider::new(&root);

        ensure_remote_layout(&provider).expect("layout should initialize");

        assert!(provider.exists(MANIFEST_PATH).expect("manifest check"));
        assert!(provider.exists(SNAPSHOTS_PATH).expect("snapshots check"));

        fs::remove_dir_all(root).ok();
    }

    #[test]
    fn snapshot_listing_only_returns_sync_packages() {
        let root =
            std::env::temp_dir().join(format!("cliply-remote-sync-test-{}", uuid::Uuid::new_v4()));
        let provider = LocalFolderSyncProvider::new(&root);
        ensure_remote_layout(&provider).expect("layout should initialize");
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

    #[test]
    fn provider_status_redacts_remote_passwords() {
        let status = RemoteSyncStatus {
            provider: SyncProviderConfig::Webdav {
                url: "https://dav.example.com".to_string(),
                username: "earl".to_string(),
                password: "secret-webdav".to_string(),
                remote_path: "cliply".to_string(),
            },
            saved_provider_configs: SavedSyncProviderConfigs {
                local_folder: None,
                webdav: Some(SyncProviderConfig::Webdav {
                    url: "https://dav.example.com".to_string(),
                    username: "earl".to_string(),
                    password: "secret-webdav".to_string(),
                    remote_path: "cliply".to_string(),
                }),
                ftp: Some(SyncProviderConfig::Ftp {
                    host: "ftp.example.com".to_string(),
                    port: 21,
                    username: "earl".to_string(),
                    password: "secret-ftp".to_string(),
                    secure: false,
                    remote_path: "cliply".to_string(),
                }),
            },
            manifest_exists: false,
            last_synced_at: None,
            last_status: None,
            last_error: None,
            snapshot_count: 0,
            auto_sync_enabled: false,
            auto_sync_interval_minutes: 5,
            sync_password_saved: false,
            sync_password_updated_at: None,
            last_auto_sync_at: None,
        };

        let status = redact_remote_sync_status(status);

        assert_eq!(remote_password(&status.provider), Some(""));
        assert_eq!(
            status
                .saved_provider_configs
                .webdav
                .as_ref()
                .and_then(remote_password),
            Some("")
        );
        assert_eq!(
            status
                .saved_provider_configs
                .ftp
                .as_ref()
                .and_then(remote_password),
            Some("")
        );
    }

    #[test]
    fn provider_password_redaction_preserves_connection_fields() {
        let redacted = redact_provider_password(SyncProviderConfig::Ftp {
            host: "ftp.example.com".to_string(),
            port: 990,
            username: "earl".to_string(),
            password: "secret".to_string(),
            secure: true,
            remote_path: "/cliply".to_string(),
        });

        assert_eq!(remote_password(&redacted), Some(""));
        match redacted {
            SyncProviderConfig::Ftp {
                host,
                port,
                username,
                secure,
                remote_path,
                ..
            } => {
                assert_eq!(host, "ftp.example.com");
                assert_eq!(port, 990);
                assert_eq!(username, "earl");
                assert!(secure);
                assert_eq!(remote_path, "/cliply");
            }
            _ => panic!("expected ftp provider"),
        }
    }

    fn remote_password(config: &SyncProviderConfig) -> Option<&str> {
        match config {
            SyncProviderConfig::Webdav { password, .. }
            | SyncProviderConfig::Ftp { password, .. } => Some(password.as_str()),
            _ => None,
        }
    }
}
