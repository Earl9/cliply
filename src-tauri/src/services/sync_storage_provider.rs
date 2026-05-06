use crate::error::CliplyError;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

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
#[serde(tag = "type", rename_all = "kebab-case")]
pub enum SyncProviderConfig {
    Disabled,
    LocalFolder {
        path: String,
    },
    Webdav {
        url: String,
        username: String,
        password: String,
        remote_path: String,
    },
    Sftp {
        host: String,
        port: u16,
        username: String,
        auth_type: String,
        remote_path: String,
    },
    Ftp {
        host: String,
        port: u16,
        username: String,
        password: String,
        secure: bool,
        remote_path: String,
    },
    S3 {
        endpoint: String,
        bucket: String,
        access_key_id: String,
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

#[cfg(test)]
mod tests {
    use super::{LocalFolderSyncProvider, SyncStorageProvider};
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
}
