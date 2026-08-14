use thiserror::Error;

#[derive(Debug, Error)]
pub enum CliplyError {
    #[error("存储服务尚未就绪：{0}")]
    StorageUnavailable(String),
    #[error("数据库错误：{0}")]
    Database(#[from] rusqlite::Error),
    #[error("文件系统错误：{0}")]
    Filesystem(#[from] std::io::Error),
    #[error("同步错误：{0}")]
    Sync(String),
    #[error("系统接口不可用：{0}")]
    PlatformUnavailable(String),
}

impl From<CliplyError> for String {
    fn from(error: CliplyError) -> Self {
        error.to_string()
    }
}
