use crate::error::CliplyError;
use serde::{Deserialize, Serialize};
use std::fs;
use std::io::Cursor;
use std::net::{SocketAddr, ToSocketAddrs};
use std::path::PathBuf;
use std::time::Duration;
use suppaftp::native_tls::TlsConnector;
use suppaftp::{FtpStream, NativeTlsConnector, NativeTlsFtpStream};

pub trait SyncStorageProvider {
    fn list(&self, path: &str) -> Result<Vec<SyncStorageEntry>, CliplyError>;
    fn read(&self, path: &str) -> Result<Vec<u8>, CliplyError>;
    fn write(&self, path: &str, data: &[u8]) -> Result<(), CliplyError>;
    fn delete(&self, path: &str) -> Result<(), CliplyError>;
    fn exists(&self, path: &str) -> Result<bool, CliplyError>;
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct SyncStorageEntry {
    pub path: String,
    pub name: String,
    pub is_dir: bool,
    pub size_bytes: u64,
    pub modified_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(
    tag = "type",
    rename_all = "kebab-case",
    rename_all_fields = "camelCase"
)]
pub enum SyncProviderConfig {
    Disabled,
    LocalFolder {
        path: String,
    },
    Webdav {
        url: String,
        username: String,
        password: String,
        #[serde(alias = "remote_path")]
        remote_path: String,
    },
    Sftp {
        host: String,
        port: u16,
        username: String,
        auth_type: String,
        #[serde(alias = "remote_path")]
        remote_path: String,
    },
    Ftp {
        host: String,
        port: u16,
        username: String,
        password: String,
        secure: bool,
        #[serde(alias = "remote_path")]
        remote_path: String,
    },
    S3 {
        endpoint: String,
        bucket: String,
        #[serde(alias = "access_key_id")]
        access_key_id: String,
        #[serde(alias = "secret_access_key")]
        secret_access_key: String,
        region: String,
        prefix: String,
    },
}

impl Default for SyncProviderConfig {
    fn default() -> Self {
        Self::Disabled
    }
}

#[derive(Debug, Clone)]
pub struct LocalFolderSyncProvider {
    root: PathBuf,
}

impl LocalFolderSyncProvider {
    pub fn new(root: impl Into<PathBuf>) -> Self {
        Self { root: root.into() }
    }

    fn resolve(&self, relative_path: &str) -> Result<PathBuf, CliplyError> {
        let normalized = relative_path.replace('\\', "/");
        let mut resolved = self.root.clone();
        for part in normalized.split('/') {
            if part.is_empty() || part == "." {
                continue;
            }
            if part == ".." {
                return Err(CliplyError::Sync("同步路径不允许访问上级目录".to_string()));
            }
            resolved.push(part);
        }
        Ok(resolved)
    }
}

impl SyncStorageProvider for LocalFolderSyncProvider {
    fn list(&self, path: &str) -> Result<Vec<SyncStorageEntry>, CliplyError> {
        let directory = self.resolve(path)?;
        if !directory.exists() {
            return Ok(Vec::new());
        }

        let mut entries = Vec::new();
        for entry in fs::read_dir(directory)? {
            let entry = entry?;
            let metadata = entry.metadata()?;
            let name = entry.file_name().to_string_lossy().to_string();
            let relative = if path.is_empty() {
                name.clone()
            } else {
                format!("{}/{}", path.trim_matches('/'), name)
            };
            entries.push(SyncStorageEntry {
                path: relative.replace('\\', "/"),
                name,
                is_dir: metadata.is_dir(),
                size_bytes: metadata.len(),
                modified_at: None,
            });
        }
        entries.sort_by(|left, right| left.path.cmp(&right.path));
        Ok(entries)
    }

    fn read(&self, path: &str) -> Result<Vec<u8>, CliplyError> {
        Ok(fs::read(self.resolve(path)?)?)
    }

    fn write(&self, path: &str, data: &[u8]) -> Result<(), CliplyError> {
        let output_path = self.resolve(path)?;
        if let Some(parent) = output_path.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::write(output_path, data)?;
        Ok(())
    }

    fn delete(&self, path: &str) -> Result<(), CliplyError> {
        let target = self.resolve(path)?;
        if target.is_dir() {
            fs::remove_dir_all(target)?;
        } else if target.exists() {
            fs::remove_file(target)?;
        }
        Ok(())
    }

    fn exists(&self, path: &str) -> Result<bool, CliplyError> {
        Ok(self.resolve(path)?.exists())
    }
}

#[derive(Debug, Clone)]
pub struct FtpSyncProvider {
    host: String,
    port: u16,
    username: String,
    password: String,
    secure: bool,
    remote_path: String,
    timeout: Duration,
}

enum FtpConnection {
    Plain(FtpStream),
    Secure(NativeTlsFtpStream),
}

impl FtpSyncProvider {
    pub fn new(
        host: String,
        port: u16,
        username: String,
        password: String,
        secure: bool,
        remote_path: String,
    ) -> Self {
        Self {
            host,
            port,
            username,
            password,
            secure,
            remote_path,
            timeout: Duration::from_secs(12),
        }
    }

    fn connect(&self) -> Result<FtpConnection, CliplyError> {
        let address = self
            .socket_addr()
            .map_err(|error| CliplyError::Sync(format!("FTP 地址解析失败: {error}")))?;

        if self.secure {
            let connector = TlsConnector::new()
                .map(NativeTlsConnector::from)
                .map_err(|error| CliplyError::Sync(format!("FTPS 初始化失败: {error}")))?;
            let stream = NativeTlsFtpStream::connect_timeout(address, self.timeout)
                .map_err(|error| CliplyError::Sync(format!("FTPS 连接失败: {error}")))?
                .into_secure(connector, &self.host)
                .map_err(|error| CliplyError::Sync(format!("FTPS 握手失败: {error}")))?;
            let mut connection = FtpConnection::Secure(stream);
            connection.login(&self.username, &self.password)?;
            return Ok(connection);
        }

        let stream = FtpStream::connect_timeout(address, self.timeout)
            .map_err(|error| CliplyError::Sync(format!("FTP 连接失败: {error}")))?;
        let mut connection = FtpConnection::Plain(stream);
        connection.login(&self.username, &self.password)?;
        Ok(connection)
    }

    fn socket_addr(&self) -> Result<SocketAddr, std::io::Error> {
        let address = format!("{}:{}", self.host.trim(), self.port);
        address
            .to_socket_addrs()?
            .next()
            .ok_or_else(|| std::io::Error::new(std::io::ErrorKind::NotFound, "no address found"))
    }

    fn connect_with_path(
        &self,
        relative_path: &str,
    ) -> Result<(FtpConnection, String), CliplyError> {
        let connection = self.connect()?;
        let remote_path = self.resolve(relative_path)?;
        Ok((connection, remote_path))
    }

    fn resolve(&self, relative_path: &str) -> Result<String, CliplyError> {
        self.resolve_with_initial_dir(relative_path, None)
    }

    fn resolve_with_initial_dir(
        &self,
        relative_path: &str,
        _initial_directory: Option<&str>,
    ) -> Result<String, CliplyError> {
        let remote_path = normalize_ftp_remote_root(&self.remote_path);
        let is_absolute = remote_path.trim_start().starts_with('/');
        let mut parts = Vec::new();
        for part in remote_path.split('/') {
            push_safe_path_part(&mut parts, part)?;
        }
        for part in relative_path.replace('\\', "/").split('/') {
            push_safe_path_part(&mut parts, part)?;
        }
        let resolved = parts.join("/");
        if is_absolute {
            Ok(format!("/{resolved}"))
        } else {
            Ok(resolved)
        }
    }

    fn ensure_parent_dirs(
        &self,
        connection: &mut FtpConnection,
        path: &str,
    ) -> Result<(), CliplyError> {
        let mut parts = path
            .split('/')
            .filter(|part| !part.is_empty())
            .collect::<Vec<_>>();
        if parts.is_empty() {
            return Ok(());
        }
        parts.pop();
        if parts.is_empty() {
            return Ok(());
        }

        let mut current = if path.starts_with('/') {
            "/".to_string()
        } else {
            String::new()
        };
        for part in parts {
            if current == "/" {
                current.push_str(part);
            } else if !current.is_empty() {
                current.push('/');
                current.push_str(part);
            } else {
                current.push_str(part);
            }
            if !connection.exists(&current)? {
                connection.mkdir(&current)?;
            }
        }
        Ok(())
    }
}

impl SyncStorageProvider for FtpSyncProvider {
    fn list(&self, path: &str) -> Result<Vec<SyncStorageEntry>, CliplyError> {
        let (mut connection, remote_path) = self.connect_with_path(path)?;
        let original_directory = connection.pwd().ok();
        let names = match connection.nlst(Some(&remote_path)) {
            Ok(names) => names,
            Err(_) => {
                connection.quit();
                return Ok(Vec::new());
            }
        };

        let mut entries = Vec::new();
        for raw_name in names {
            let name = raw_name
                .trim_end_matches('/')
                .rsplit('/')
                .next()
                .unwrap_or(raw_name.as_str())
                .to_string();
            if name.is_empty() || name == "." || name == ".." {
                continue;
            }

            let child_path = if path.trim_matches('/').is_empty() {
                name.clone()
            } else {
                format!("{}/{}", path.trim_matches('/'), name)
            };
            let remote_child = self.resolve(&child_path)?;
            let is_dir = connection.cwd(&remote_child).is_ok();
            if is_dir {
                connection.restore_working_dir(original_directory.as_deref());
            }
            let size_bytes = if is_dir {
                0
            } else {
                connection.size(&remote_child).unwrap_or(0) as u64
            };
            entries.push(SyncStorageEntry {
                path: child_path.replace('\\', "/"),
                name,
                is_dir,
                size_bytes,
                modified_at: None,
            });
        }
        connection.quit();
        entries.sort_by(|left, right| left.path.cmp(&right.path));
        Ok(entries)
    }

    fn read(&self, path: &str) -> Result<Vec<u8>, CliplyError> {
        let (mut connection, remote_path) = self.connect_with_path(path)?;
        let bytes = connection
            .retr_as_buffer(&remote_path)
            .map_err(|error| CliplyError::Sync(format!("FTP 读取失败: {error}")))?
            .into_inner();
        connection.quit();
        Ok(bytes)
    }

    fn write(&self, path: &str, data: &[u8]) -> Result<(), CliplyError> {
        let (mut connection, remote_path) = self.connect_with_path(path)?;
        self.ensure_parent_dirs(&mut connection, &remote_path)?;
        let mut cursor = Cursor::new(data);
        connection
            .put_file(&remote_path, &mut cursor)
            .map_err(|error| CliplyError::Sync(format!("FTP 写入失败: {error}")))?;
        connection.quit();
        Ok(())
    }

    fn delete(&self, path: &str) -> Result<(), CliplyError> {
        let (mut connection, remote_path) = self.connect_with_path(path)?;
        if connection.rm(&remote_path).is_err() {
            let _ = connection.rmdir(&remote_path);
        }
        connection.quit();
        Ok(())
    }

    fn exists(&self, path: &str) -> Result<bool, CliplyError> {
        let (mut connection, remote_path) = self.connect_with_path(path)?;
        let exists = connection.exists(&remote_path)?;
        connection.quit();
        Ok(exists)
    }
}

impl FtpConnection {
    fn login(&mut self, username: &str, password: &str) -> Result<(), CliplyError> {
        match self {
            Self::Plain(stream) => stream.login(username, password),
            Self::Secure(stream) => stream.login(username, password),
        }
        .map_err(|error| CliplyError::Sync(format!("FTP 登录失败: {error}")))
    }

    fn nlst(&mut self, path: Option<&str>) -> Result<Vec<String>, suppaftp::FtpError> {
        match self {
            Self::Plain(stream) => stream.nlst(path),
            Self::Secure(stream) => stream.nlst(path),
        }
    }

    fn retr_as_buffer(&mut self, path: &str) -> Result<Cursor<Vec<u8>>, suppaftp::FtpError> {
        match self {
            Self::Plain(stream) => stream.retr_as_buffer(path),
            Self::Secure(stream) => stream.retr_as_buffer(path),
        }
    }

    fn put_file(
        &mut self,
        path: &str,
        data: &mut Cursor<&[u8]>,
    ) -> Result<u64, suppaftp::FtpError> {
        match self {
            Self::Plain(stream) => stream.put_file(path, data),
            Self::Secure(stream) => stream.put_file(path, data),
        }
    }

    fn mkdir(&mut self, path: &str) -> Result<(), CliplyError> {
        match self {
            Self::Plain(stream) => stream.mkdir(path),
            Self::Secure(stream) => stream.mkdir(path),
        }
        .or_else(|_| Ok(()))
    }

    fn cwd(&mut self, path: &str) -> Result<(), suppaftp::FtpError> {
        match self {
            Self::Plain(stream) => stream.cwd(path),
            Self::Secure(stream) => stream.cwd(path),
        }
    }

    fn cdup(&mut self) -> Result<(), suppaftp::FtpError> {
        match self {
            Self::Plain(stream) => stream.cdup(),
            Self::Secure(stream) => stream.cdup(),
        }
    }

    fn pwd(&mut self) -> Result<String, suppaftp::FtpError> {
        match self {
            Self::Plain(stream) => stream.pwd(),
            Self::Secure(stream) => stream.pwd(),
        }
    }

    fn restore_working_dir(&mut self, path: Option<&str>) {
        if let Some(path) = path {
            let _ = self.cwd(path);
        } else {
            let _ = self.cdup();
        }
    }

    fn size(&mut self, path: &str) -> Result<usize, suppaftp::FtpError> {
        match self {
            Self::Plain(stream) => stream.size(path),
            Self::Secure(stream) => stream.size(path),
        }
    }

    fn rm(&mut self, path: &str) -> Result<(), suppaftp::FtpError> {
        match self {
            Self::Plain(stream) => stream.rm(path),
            Self::Secure(stream) => stream.rm(path),
        }
    }

    fn rmdir(&mut self, path: &str) -> Result<(), suppaftp::FtpError> {
        match self {
            Self::Plain(stream) => stream.rmdir(path),
            Self::Secure(stream) => stream.rmdir(path),
        }
    }

    fn exists(&mut self, path: &str) -> Result<bool, CliplyError> {
        if self.size(path).is_ok() {
            return Ok(true);
        }
        let original_directory = self.pwd().ok();
        if self.cwd(path).is_ok() {
            self.restore_working_dir(original_directory.as_deref());
            return Ok(true);
        }
        Ok(false)
    }

    fn quit(&mut self) {
        let _ = match self {
            Self::Plain(stream) => stream.quit(),
            Self::Secure(stream) => stream.quit(),
        };
    }
}

fn push_safe_path_part(parts: &mut Vec<String>, part: &str) -> Result<(), CliplyError> {
    let part = part.trim();
    if part.is_empty() || part == "." {
        return Ok(());
    }
    if part == ".." || part.contains('\\') {
        return Err(CliplyError::Sync("同步路径不允许访问上级目录".to_string()));
    }
    parts.push(part.to_string());
    Ok(())
}

fn normalize_ftp_remote_root(path: &str) -> String {
    let normalized = path.replace('\\', "/").trim().to_string();
    if let Some(path) = normalized.strip_prefix("mnt/") {
        return format!("/mnt/{path}");
    }
    normalized
}

#[cfg(test)]
mod tests {
    use super::{
        FtpSyncProvider, LocalFolderSyncProvider, SyncProviderConfig, SyncStorageProvider,
    };
    use std::fs;

    #[test]
    fn local_folder_provider_roundtrips_data() {
        let root = std::env::temp_dir().join(format!(
            "cliply-sync-provider-test-{}",
            uuid::Uuid::new_v4()
        ));
        let provider = LocalFolderSyncProvider::new(&root);

        provider
            .write("CliplySync/snapshots/test.cliply-sync", b"encrypted")
            .expect("write should succeed");
        assert!(provider
            .exists("CliplySync/snapshots/test.cliply-sync")
            .expect("exists should run"));
        assert_eq!(
            provider
                .read("CliplySync/snapshots/test.cliply-sync")
                .expect("read should succeed"),
            b"encrypted"
        );
        assert_eq!(
            provider
                .list("CliplySync/snapshots")
                .expect("list should succeed")
                .len(),
            1
        );
        provider
            .delete("CliplySync/snapshots/test.cliply-sync")
            .expect("delete should succeed");
        assert!(!provider
            .exists("CliplySync/snapshots/test.cliply-sync")
            .expect("exists should run"));

        fs::remove_dir_all(root).ok();
    }

    #[test]
    fn local_folder_rejects_parent_traversal() {
        let provider = LocalFolderSyncProvider::new(std::env::temp_dir());
        let error = provider
            .write("../outside.cliply-sync", b"bad")
            .expect_err("parent traversal should fail");
        assert!(error.to_string().contains("上级目录"));
    }

    #[test]
    fn ftp_provider_normalizes_remote_paths() {
        let provider = FtpSyncProvider::new(
            "example.com".to_string(),
            21,
            "user".to_string(),
            "password".to_string(),
            false,
            "/cliply/./remote/".to_string(),
        );

        assert_eq!(
            provider
                .resolve("CliplySync\\snapshots/test.cliply-sync")
                .expect("path should resolve"),
            "/cliply/remote/CliplySync/snapshots/test.cliply-sync"
        );
    }

    #[test]
    fn ftp_provider_resolves_absolute_remote_paths() {
        let provider = FtpSyncProvider::new(
            "example.com".to_string(),
            21,
            "user".to_string(),
            "password".to_string(),
            false,
            "/user/sync/cliply".to_string(),
        );

        assert_eq!(
            provider
                .resolve("CliplySync/snapshots/test.cliply-sync")
                .expect("path should resolve"),
            "/user/sync/cliply/CliplySync/snapshots/test.cliply-sync"
        );
    }

    #[test]
    fn ftp_provider_keeps_absolute_mnt_paths() {
        let provider = FtpSyncProvider::new(
            "example.com".to_string(),
            21,
            "user".to_string(),
            "password".to_string(),
            false,
            "/mnt/user/sync/cliply".to_string(),
        );

        assert_eq!(
            provider
                .resolve_with_initial_dir("CliplySync/snapshots/test.cliply-sync", Some("/mnt"))
                .expect("path should resolve"),
            "/mnt/user/sync/cliply/CliplySync/snapshots/test.cliply-sync"
        );
    }

    #[test]
    fn ftp_provider_repairs_common_mnt_paths_without_leading_slash() {
        let provider = FtpSyncProvider::new(
            "example.com".to_string(),
            21,
            "user".to_string(),
            "password".to_string(),
            false,
            "mnt/user/sync/cliply".to_string(),
        );

        assert_eq!(
            provider
                .resolve_with_initial_dir("CliplySync/snapshots/test.cliply-sync", Some("/"))
                .expect("path should resolve"),
            "/mnt/user/sync/cliply/CliplySync/snapshots/test.cliply-sync"
        );
    }

    #[test]
    fn ftp_provider_rejects_parent_traversal() {
        let provider = FtpSyncProvider::new(
            "example.com".to_string(),
            21,
            "user".to_string(),
            "password".to_string(),
            false,
            "cliply".to_string(),
        );

        let error = provider
            .resolve("CliplySync/../manifest.json")
            .expect_err("parent traversal should fail");
        assert!(error.to_string().contains("上级目录"));
    }

    #[test]
    fn ftp_provider_config_uses_camel_case_json_and_accepts_legacy_snake_case() {
        let config = SyncProviderConfig::Ftp {
            host: "10.0.0.9".to_string(),
            port: 21,
            username: "root".to_string(),
            password: "secret".to_string(),
            secure: false,
            remote_path: "/mnt/user/sync/cliply".to_string(),
        };

        let json = serde_json::to_string(&config).expect("config should serialize");
        assert!(json.contains("\"remotePath\""));
        assert!(!json.contains("\"remote_path\""));

        let parsed: SyncProviderConfig = serde_json::from_str(
            r#"{
                "type": "ftp",
                "host": "10.0.0.9",
                "port": 21,
                "username": "root",
                "password": "secret",
                "secure": false,
                "remotePath": "/mnt/user/sync/cliply"
            }"#,
        )
        .expect("camelCase config should deserialize");
        assert_eq!(parsed, config);

        let legacy: SyncProviderConfig = serde_json::from_str(
            r#"{
                "type": "ftp",
                "host": "10.0.0.9",
                "port": 21,
                "username": "root",
                "password": "secret",
                "secure": false,
                "remote_path": "/mnt/user/sync/cliply"
            }"#,
        )
        .expect("snake_case config should deserialize");
        assert_eq!(legacy, config);
    }

    #[test]
    #[ignore = "requires a real FTP server configured with CLIPLY_TEST_FTP_* env vars"]
    fn ftp_provider_roundtrips_against_configured_server() {
        let host = std::env::var("CLIPLY_TEST_FTP_HOST").expect("CLIPLY_TEST_FTP_HOST is required");
        let port = std::env::var("CLIPLY_TEST_FTP_PORT")
            .ok()
            .and_then(|value| value.parse::<u16>().ok())
            .unwrap_or(21);
        let username =
            std::env::var("CLIPLY_TEST_FTP_USER").expect("CLIPLY_TEST_FTP_USER is required");
        let password = std::env::var("CLIPLY_TEST_FTP_PASSWORD")
            .expect("CLIPLY_TEST_FTP_PASSWORD is required");
        let secure = std::env::var("CLIPLY_TEST_FTP_SECURE")
            .map(|value| value == "1" || value.eq_ignore_ascii_case("true"))
            .unwrap_or(false);
        let remote_path =
            std::env::var("CLIPLY_TEST_FTP_REMOTE_PATH").unwrap_or_else(|_| "cliply".to_string());
        let provider = FtpSyncProvider::new(host, port, username, password, secure, remote_path);
        let file_name = format!("CliplySync/ftp-provider-test-{}.tmp", uuid::Uuid::new_v4());
        let payload = b"cliply ftp provider test";

        provider
            .write(&file_name, payload)
            .expect("write should succeed");
        assert!(provider.exists(&file_name).expect("exists should run"));
        assert_eq!(
            provider.read(&file_name).expect("read should succeed"),
            payload
        );
        assert!(provider
            .list("CliplySync")
            .expect("list should succeed")
            .iter()
            .any(|entry| entry.path == file_name));
        provider.delete(&file_name).expect("delete should succeed");
        assert!(!provider.exists(&file_name).expect("exists should run"));
    }
}
