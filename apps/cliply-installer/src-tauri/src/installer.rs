use serde::{Deserialize, Serialize};
use std::{
    fs, io,
    io::Read,
    path::{Path, PathBuf},
    process::Command,
    thread,
    time::{Duration, Instant},
};
use thiserror::Error;

use crate::{payload, platform};

const PRODUCT_NAME: &str = "Cliply";
const PRODUCT_EXE: &str = "cliply.exe";
const PRODUCT_ICON: &str = concat!("cliply-", env!("CARGO_PKG_VERSION"), ".ico");
const PRODUCT_UNINSTALLER: &str = "uninstall.exe";
const PRODUCT_REG_KEY: &str = r"Software\cliply\Cliply";
const PRODUCT_UNINSTALL_KEY: &str = r"Software\Microsoft\Windows\CurrentVersion\Uninstall\Cliply";
const START_MENU_FOLDER: &str = "Cliply";
const PARENT_EXIT_TIMEOUT: Duration = Duration::from_secs(12);
const PROCESS_EXIT_TIMEOUT: Duration = Duration::from_secs(8);
const FILE_REMOVE_TIMEOUT: Duration = Duration::from_secs(15);
const FILE_REPLACE_TIMEOUT: Duration = Duration::from_secs(30);

#[derive(Debug, Error)]
pub enum InstallError {
    #[error("请选择安装目录")]
    EmptyInstallDir,
    #[error("安装目录无效：{0}")]
    InvalidInstallDir(String),
    #[error("无法创建目录 {path}：{source}")]
    CreateDir {
        path: String,
        source: std::io::Error,
    },
    #[error("无法写入文件 {path}：{source}")]
    WriteFile {
        path: String,
        source: std::io::Error,
    },
    #[error(
        "程序文件仍被占用，更新未完成。请关闭 Cliply 后重试；如仍失败，请重启 Windows。"
    )]
    ReplaceLockedFile {
        path: String,
        source: std::io::Error,
    },
    #[error("无法关闭 Cliply。请从系统托盘退出 Cliply，然后重试。")]
    StopRunningCliply,
    #[error("无法启动 Cliply：{0}")]
    Launch(std::io::Error),
    #[error("无法读取安装程序路径：{0}")]
    CurrentExe(std::io::Error),
    #[error("无法解压 Cliply 程序文件：{0}")]
    Decompress(std::io::Error),
    #[error("{0}")]
    Platform(String),
}

impl From<platform::PlatformError> for InstallError {
    fn from(error: platform::PlatformError) -> Self {
        Self::Platform(error.to_string())
    }
}

pub type InstallResult<T> = Result<T, InstallError>;

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct InstallDetection {
    pub is_update: bool,
    pub install_dir: String,
    pub existing_install_dir: Option<String>,
    pub default_install_dir: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InstallOptions {
    pub install_dir: String,
    pub create_desktop_shortcut: bool,
    pub start_on_login: bool,
    pub is_update: bool,
    pub preserve_user_data: bool,
    pub launch_after_install: bool,
    pub parent_pid: Option<u32>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct InstallOutcome {
    pub install_dir: String,
    pub is_update: bool,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct InstallerMode {
    pub is_uninstall: bool,
    pub is_update: bool,
    pub install_dir: Option<String>,
    pub source_version: Option<String>,
    pub target_version: Option<String>,
    pub preserve_user_data: bool,
    pub launch_after_install: bool,
    pub parent_pid: Option<u32>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UninstallOptions {
    pub install_dir: String,
    pub remove_user_data: bool,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UninstallOutcome {
    pub install_dir: String,
    pub user_data_removed: bool,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct InstallProgress {
    pub progress: u8,
    pub step: String,
}

pub fn detect_installation() -> InstallResult<InstallDetection> {
    let default_install_dir = default_install_dir();
    let existing_install_dir = platform::read_install_dir_from_registry()
        .or_else(detect_current_uninstall_dir)
        .or_else(|| detect_default_install_dir(&default_install_dir));
    let install_dir = existing_install_dir
        .clone()
        .unwrap_or_else(|| default_install_dir.clone());

    Ok(InstallDetection {
        is_update: existing_install_dir.is_some(),
        install_dir,
        existing_install_dir,
        default_install_dir,
    })
}

pub fn detect_mode() -> InstallerMode {
    let args: Vec<String> = std::env::args().collect();
    let is_uninstall = args.iter().any(|arg| arg == "--uninstall") || running_as_uninstaller();
    let is_update = !is_uninstall
        && (arg_value(&args, "--mode").is_some_and(|value| value.eq_ignore_ascii_case("update"))
            || args.iter().any(|arg| arg == "--update"));

    InstallerMode {
        is_uninstall,
        is_update,
        install_dir: arg_value(&args, "--install-dir"),
        source_version: arg_value(&args, "--source-version"),
        target_version: arg_value(&args, "--target-version"),
        preserve_user_data: args.iter().any(|arg| arg == "--preserve-user-data"),
        launch_after_install: args.iter().any(|arg| arg == "--launch-after-install"),
        parent_pid: arg_value(&args, "--parent-pid").and_then(|value| value.parse().ok()),
    }
}

fn running_as_uninstaller() -> bool {
    std::env::current_exe()
        .ok()
        .and_then(|path| {
            path.file_name()
                .map(|name| name.to_string_lossy().to_string())
        })
        .is_some_and(|name| name.eq_ignore_ascii_case(PRODUCT_UNINSTALLER))
}

pub fn install<F>(options: InstallOptions, mut on_progress: F) -> InstallResult<InstallOutcome>
where
    F: FnMut(InstallProgress),
{
    let install_dir = normalize_install_dir(&options.install_dir)?;
    let preserve_user_data = options.preserve_user_data || options.is_update;
    if let Some(parent_pid) = options.parent_pid {
        on_progress(progress(4, "正在等待 Cliply 退出"));
        if !wait_for_process_exit(parent_pid, PARENT_EXIT_TIMEOUT) {
            terminate_process_tree(parent_pid);
            if !wait_for_process_exit(parent_pid, PROCESS_EXIT_TIMEOUT) {
                return Err(InstallError::StopRunningCliply);
            }
        }
    }

    on_progress(progress(8, "正在关闭 Cliply"));
    stop_running_cliply()?;

    on_progress(progress(24, "正在复制 Cliply 程序文件"));
    write_payload(&install_dir)?;

    let exe_path = install_dir.join(PRODUCT_EXE);
    let icon_path = install_dir.join(PRODUCT_ICON);
    on_progress(progress(55, "正在写入安装信息"));
    platform::write_install_registry(
        PRODUCT_NAME,
        PRODUCT_REG_KEY,
        PRODUCT_UNINSTALL_KEY,
        &install_dir,
        &icon_path,
    )?;

    on_progress(progress(72, "正在创建开始菜单快捷方式"));
    platform::create_start_menu_shortcuts(START_MENU_FOLDER, &exe_path, &icon_path)?;

    on_progress(progress(86, "正在应用安装选项"));
    if options.is_update {
        platform::refresh_desktop_shortcut_if_exists(&exe_path, &icon_path)?;
        platform::refresh_start_on_login_if_enabled(PRODUCT_NAME, &exe_path)?;
    } else {
        if options.create_desktop_shortcut {
            platform::create_desktop_shortcut(&exe_path, &icon_path)?;
        } else {
            let _ = platform::remove_desktop_shortcut();
        }

        platform::set_start_on_login(PRODUCT_NAME, &exe_path, options.start_on_login)?;
    }

    let _ = remove_stale_product_icons(&install_dir, Some(&icon_path));
    refresh_shell_icon_cache();
    on_progress(progress(
        100,
        if options.is_update {
            "Cliply 更新完成"
        } else if preserve_user_data {
            "Cliply 安装完成，剪贴板历史记录和应用设置已保留"
        } else {
            "Cliply 安装完成"
        },
    ));

    if options.launch_after_install {
        let _ = launch_cliply(install_dir.to_string_lossy().to_string());
    }

    Ok(InstallOutcome {
        install_dir: install_dir.to_string_lossy().to_string(),
        is_update: options.is_update,
    })
}

pub fn launch_cliply(install_dir: String) -> InstallResult<()> {
    let install_dir = normalize_install_dir(&install_dir)?;
    Command::new(install_dir.join(PRODUCT_EXE))
        .spawn()
        .map(|_| ())
        .map_err(InstallError::Launch)
}

pub fn open_installer_log_directory() -> InstallResult<()> {
    let dir = user_data_dirs()
        .into_iter()
        .next()
        .unwrap_or_else(|| PathBuf::from(r"C:\ProgramData\Cliply"));
    fs::create_dir_all(&dir).map_err(|source| InstallError::CreateDir {
        path: dir.to_string_lossy().to_string(),
        source,
    })?;
    open_path(&dir).map_err(InstallError::Launch)
}

pub fn open_release_page() -> InstallResult<()> {
    open_url("https://github.com/Earl9/cliply/releases/latest").map_err(InstallError::Launch)
}

pub fn uninstall<F>(
    options: UninstallOptions,
    mut on_progress: F,
) -> InstallResult<UninstallOutcome>
where
    F: FnMut(InstallProgress),
{
    let install_dir = normalize_install_dir(&options.install_dir)?;
    on_progress(progress(10, "正在关闭 Cliply"));
    stop_running_cliply()?;

    on_progress(progress(28, "正在移除快捷方式和开机启动项"));
    let exe_path = install_dir.join(PRODUCT_EXE);
    let _ = platform::set_start_on_login(PRODUCT_NAME, &exe_path, false);
    let _ = platform::remove_desktop_shortcut();
    let _ = platform::remove_start_menu_shortcuts(START_MENU_FOLDER);

    on_progress(progress(52, "正在删除 Cliply 程序文件"));
    remove_installed_files(&install_dir)?;

    on_progress(progress(72, "正在清理安装信息"));
    platform::remove_install_registry(PRODUCT_REG_KEY, PRODUCT_UNINSTALL_KEY)?;
    refresh_shell_icon_cache();

    if options.remove_user_data {
        on_progress(progress(88, "正在删除剪贴板历史记录和应用设置"));
        remove_user_data()?;
    }

    schedule_self_delete_if_needed(&install_dir);
    on_progress(progress(100, "卸载完成"));

    Ok(UninstallOutcome {
        install_dir: install_dir.to_string_lossy().to_string(),
        user_data_removed: options.remove_user_data,
    })
}

fn write_payload(install_dir: &Path) -> InstallResult<()> {
    fs::create_dir_all(install_dir).map_err(|source| InstallError::CreateDir {
        path: install_dir.to_string_lossy().to_string(),
        source,
    })?;

    let exe_bytes = decompress_payload(payload::CLIPLY_EXE_GZ)?;
    write_file_atomic(&install_dir.join(PRODUCT_EXE), &exe_bytes)?;
    write_file_atomic(&install_dir.join(PRODUCT_ICON), payload::CLIPLY_ICON)?;
    write_uninstaller(install_dir)?;
    Ok(())
}

fn decompress_payload(bytes: &[u8]) -> InstallResult<Vec<u8>> {
    let mut decoder = flate2::read::GzDecoder::new(bytes);
    let mut decoded = Vec::new();
    decoder
        .read_to_end(&mut decoded)
        .map_err(InstallError::Decompress)?;
    Ok(decoded)
}

fn write_uninstaller(install_dir: &Path) -> InstallResult<()> {
    let current_exe = std::env::current_exe().map_err(InstallError::CurrentExe)?;
    let bytes = fs::read(&current_exe).map_err(|source| InstallError::WriteFile {
        path: current_exe.to_string_lossy().to_string(),
        source,
    })?;
    write_file_atomic(&install_dir.join(PRODUCT_UNINSTALLER), &bytes)
}

fn remove_installed_files(install_dir: &Path) -> InstallResult<()> {
    let files = [PRODUCT_EXE, "Uninstall Cliply.lnk", "卸载 Cliply.lnk"];

    for file in files {
        let path = install_dir.join(file);
        if path.exists() {
            remove_file_with_retry(&path)?;
        }
    }

    remove_stale_product_icons(install_dir, None)?;

    Ok(())
}

fn remove_stale_product_icons(install_dir: &Path, keep: Option<&Path>) -> InstallResult<()> {
    let entries = match fs::read_dir(install_dir) {
        Ok(entries) => entries,
        Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(()),
        Err(source) => {
            return Err(InstallError::ReplaceLockedFile {
                path: install_dir.to_string_lossy().to_string(),
                source,
            })
        }
    };

    for entry in entries.flatten() {
        let path = entry.path();
        if keep.is_some_and(|keep_path| paths_equal(&path, keep_path)) {
            continue;
        }

        let Some(file_name) = path.file_name().and_then(|name| name.to_str()) else {
            continue;
        };
        let is_legacy_icon = file_name.eq_ignore_ascii_case("cliply.ico");
        let is_versioned_icon = file_name
            .to_ascii_lowercase()
            .strip_prefix("cliply-")
            .is_some_and(|suffix| suffix.ends_with(".ico"));
        if is_legacy_icon || is_versioned_icon {
            remove_file_with_retry(&path)?;
        }
    }

    Ok(())
}

fn refresh_shell_icon_cache() {
    let _ = Command::new("ie4uinit.exe")
        .arg("-show")
        .creation_flags_no_window()
        .status();
}

fn remove_user_data() -> InstallResult<()> {
    for dir in user_data_dirs() {
        if dir.exists() {
            fs::remove_dir_all(&dir).map_err(|source| InstallError::ReplaceLockedFile {
                path: dir.to_string_lossy().to_string(),
                source,
            })?;
        }
    }

    Ok(())
}

fn user_data_dirs() -> Vec<PathBuf> {
    let mut dirs = Vec::new();
    if let Some(appdata) = std::env::var_os("APPDATA") {
        dirs.push(PathBuf::from(appdata).join("com.cliply.app"));
    }
    if let Some(local_appdata) = std::env::var_os("LOCALAPPDATA") {
        dirs.push(PathBuf::from(local_appdata).join("com.cliply.app"));
    }
    dirs
}

fn paths_equal(left: &Path, right: &Path) -> bool {
    let left = left.canonicalize().unwrap_or_else(|_| left.to_path_buf());
    let right = right.canonicalize().unwrap_or_else(|_| right.to_path_buf());
    left == right
}

fn schedule_self_delete_if_needed(install_dir: &Path) {
    let Ok(current_exe) = std::env::current_exe() else {
        return;
    };
    let uninstall_exe = install_dir.join(PRODUCT_UNINSTALLER);
    if !paths_equal(&current_exe, &uninstall_exe) {
        return;
    }

    let command = format!(
        "ping 127.0.0.1 -n 3 > NUL & del /F /Q \"{}\" & rmdir \"{}\" 2> NUL",
        current_exe.to_string_lossy(),
        install_dir.to_string_lossy()
    );

    let _ = Command::new("cmd")
        .args(["/C", &command])
        .creation_flags_no_window()
        .spawn();
}

fn write_file_atomic(path: &Path, bytes: &[u8]) -> InstallResult<()> {
    let temp_path = path.with_extension("cliply-installer-new");

    if temp_path.exists() {
        remove_file_with_retry(&temp_path)?;
    }

    fs::write(&temp_path, bytes).map_err(|source| InstallError::WriteFile {
        path: temp_path.to_string_lossy().to_string(),
        source,
    })?;

    replace_file_with_retry(&temp_path, path, FILE_REPLACE_TIMEOUT).inspect_err(|_| {
        let _ = fs::remove_file(&temp_path);
    })
}

fn terminate_process_tree(pid: u32) {
    let pid = pid.to_string();
    let _ = Command::new("taskkill")
        .args(["/PID", &pid, "/F", "/T"])
        .creation_flags_no_window()
        .status();
}

fn stop_running_cliply() -> InstallResult<()> {
    if !is_cliply_running() {
        return Ok(());
    }

    let _ = Command::new("taskkill")
        .args(["/IM", PRODUCT_EXE, "/F", "/T"])
        .creation_flags_no_window()
        .status();

    wait_until_cliply_exits(PROCESS_EXIT_TIMEOUT)
        .then_some(())
        .ok_or(InstallError::StopRunningCliply)
}

fn wait_until_cliply_exits(timeout: Duration) -> bool {
    let deadline = Instant::now() + timeout;
    while Instant::now() < deadline {
        if !is_cliply_running() {
            return true;
        }

        thread::sleep(Duration::from_millis(250));
    }

    !is_cliply_running()
}

fn wait_for_process_exit(pid: u32, timeout: Duration) -> bool {
    let deadline = Instant::now() + timeout;
    while Instant::now() < deadline {
        if !is_process_running(pid) {
            return true;
        }

        thread::sleep(Duration::from_millis(250));
    }

    !is_process_running(pid)
}

fn is_process_running(pid: u32) -> bool {
    let pid_text = pid.to_string();
    let output = Command::new("tasklist")
        .args(["/FI", &format!("PID eq {pid}")])
        .creation_flags_no_window()
        .output();

    output
        .ok()
        .map(|output| contains_ascii_case_insensitive(&output.stdout, pid_text.as_bytes()))
        .unwrap_or(false)
}

fn is_cliply_running() -> bool {
    let output = Command::new("tasklist")
        .args(["/FI", &format!("IMAGENAME eq {PRODUCT_EXE}")])
        .creation_flags_no_window()
        .output();

    output
        .ok()
        .map(|output| contains_ascii_case_insensitive(&output.stdout, PRODUCT_EXE.as_bytes()))
        .unwrap_or(false)
}

fn contains_ascii_case_insensitive(haystack: &[u8], needle: &[u8]) -> bool {
    if needle.is_empty() || needle.len() > haystack.len() {
        return false;
    }

    haystack
        .windows(needle.len())
        .any(|window| window.eq_ignore_ascii_case(needle))
}

fn arg_value(args: &[String], key: &str) -> Option<String> {
    args.windows(2)
        .find(|pair| pair[0] == key)
        .map(|pair| pair[1].clone())
        .filter(|value| !value.trim().is_empty())
}

fn open_path(path: &Path) -> std::io::Result<()> {
    #[cfg(target_os = "windows")]
    {
        Command::new("explorer")
            .arg(path)
            .creation_flags_no_window()
            .spawn()
            .map(|_| ())
    }

    #[cfg(target_os = "macos")]
    {
        Command::new("open").arg(path).spawn().map(|_| ())
    }

    #[cfg(all(unix, not(target_os = "macos")))]
    {
        Command::new("xdg-open").arg(path).spawn().map(|_| ())
    }
}

fn open_url(url: &str) -> std::io::Result<()> {
    #[cfg(target_os = "windows")]
    {
        Command::new("explorer")
            .arg(url)
            .creation_flags_no_window()
            .spawn()
            .map(|_| ())
    }

    #[cfg(target_os = "macos")]
    {
        Command::new("open").arg(url).spawn().map(|_| ())
    }

    #[cfg(all(unix, not(target_os = "macos")))]
    {
        Command::new("xdg-open").arg(url).spawn().map(|_| ())
    }
}

fn remove_file_with_retry(path: &Path) -> InstallResult<()> {
    clear_readonly(path);

    let deadline = Instant::now() + FILE_REMOVE_TIMEOUT;
    let mut retry_delay = Duration::from_millis(200);
    let last_error = loop {
        let error = match fs::remove_file(path) {
            Ok(()) => return Ok(()),
            Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(()),
            Err(error) => error,
        };

        if !sleep_before_retry(deadline, &mut retry_delay) {
            break error;
        }
    };

    Err(InstallError::ReplaceLockedFile {
        path: path.to_string_lossy().to_string(),
        source: last_error,
    })
}

fn replace_file_with_retry(from: &Path, to: &Path, timeout: Duration) -> InstallResult<()> {
    clear_readonly(to);

    let deadline = Instant::now() + timeout;
    let mut retry_delay = Duration::from_millis(200);
    let last_error = loop {
        let error = match replace_file_once(from, to) {
            Ok(()) => return Ok(()),
            Err(error) => error,
        };

        if !sleep_before_retry(deadline, &mut retry_delay) {
            break error;
        }
    };

    Err(InstallError::ReplaceLockedFile {
        path: to.to_string_lossy().to_string(),
        source: last_error,
    })
}

fn sleep_before_retry(deadline: Instant, retry_delay: &mut Duration) -> bool {
    let remaining = deadline.saturating_duration_since(Instant::now());
    if remaining.is_zero() {
        return false;
    }

    thread::sleep((*retry_delay).min(remaining));
    *retry_delay = (*retry_delay + Duration::from_millis(150)).min(Duration::from_secs(1));
    true
}

#[cfg(windows)]
fn replace_file_once(from: &Path, to: &Path) -> io::Result<()> {
    use std::os::windows::ffi::OsStrExt;
    use windows::{
        core::PCWSTR,
        Win32::Storage::FileSystem::{
            MoveFileExW, MOVEFILE_REPLACE_EXISTING, MOVEFILE_WRITE_THROUGH,
        },
    };

    let from = from
        .as_os_str()
        .encode_wide()
        .chain(std::iter::once(0))
        .collect::<Vec<_>>();
    let to = to
        .as_os_str()
        .encode_wide()
        .chain(std::iter::once(0))
        .collect::<Vec<_>>();
    unsafe {
        MoveFileExW(
            PCWSTR(from.as_ptr()),
            PCWSTR(to.as_ptr()),
            MOVEFILE_REPLACE_EXISTING | MOVEFILE_WRITE_THROUGH,
        )
    }
    .map_err(|error| {
        let raw_code = (error.code().0 as u32 & 0xffff) as i32;
        if raw_code == 0 {
            io::Error::other(error.to_string())
        } else {
            io::Error::from_raw_os_error(raw_code)
        }
    })
}

#[cfg(not(windows))]
fn replace_file_once(from: &Path, to: &Path) -> io::Result<()> {
    if to.exists() {
        fs::remove_file(to)?;
    }
    fs::rename(from, to)
}

fn clear_readonly(path: &Path) {
    let Ok(metadata) = fs::metadata(path) else {
        return;
    };

    let mut permissions = metadata.permissions();
    if permissions.readonly() {
        permissions.set_readonly(false);
        let _ = fs::set_permissions(path, permissions);
    }
}

fn normalize_install_dir(value: &str) -> InstallResult<PathBuf> {
    let trimmed = value.trim().trim_matches('"');
    if trimmed.is_empty() {
        return Err(InstallError::EmptyInstallDir);
    }

    let path = PathBuf::from(trimmed);
    if path
        .components()
        .any(|component| matches!(component, std::path::Component::ParentDir))
    {
        return Err(InstallError::InvalidInstallDir(trimmed.to_string()));
    }

    Ok(path)
}

fn detect_default_install_dir(default_install_dir: &str) -> Option<String> {
    let path = Path::new(default_install_dir).join(PRODUCT_EXE);
    path.exists().then(|| default_install_dir.to_string())
}

fn detect_current_uninstall_dir() -> Option<String> {
    let current_exe = std::env::current_exe().ok()?;
    let file_name = current_exe.file_name()?.to_string_lossy();
    if !file_name.eq_ignore_ascii_case(PRODUCT_UNINSTALLER) {
        return None;
    }

    current_exe
        .parent()
        .filter(|dir| dir.join(PRODUCT_EXE).exists())
        .map(|dir| dir.to_string_lossy().to_string())
}

fn default_install_dir() -> String {
    std::env::var("ProgramFiles")
        .map(|program_files| Path::new(&program_files).join("Cliply"))
        .unwrap_or_else(|_| PathBuf::from(r"C:\Program Files\Cliply"))
        .to_string_lossy()
        .to_string()
}

fn progress(progress: u8, step: &str) -> InstallProgress {
    InstallProgress {
        progress,
        step: step.to_string(),
    }
}

trait CommandNoWindow {
    fn creation_flags_no_window(&mut self) -> &mut Self;
}

impl CommandNoWindow for Command {
    #[cfg(windows)]
    fn creation_flags_no_window(&mut self) -> &mut Self {
        use std::os::windows::process::CommandExt;
        const CREATE_NO_WINDOW: u32 = 0x08000000;
        self.creation_flags(CREATE_NO_WINDOW)
    }

    #[cfg(not(windows))]
    fn creation_flags_no_window(&mut self) -> &mut Self {
        self
    }
}

#[cfg(all(test, windows))]
mod tests {
    use super::*;
    use std::{
        fs::OpenOptions,
        os::windows::fs::OpenOptionsExt,
        time::{SystemTime, UNIX_EPOCH},
    };

    const FILE_SHARE_READ: u32 = 0x0000_0001;

    #[test]
    fn atomic_replacement_waits_for_a_transient_file_lock() {
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system clock should be valid")
            .as_nanos();
        let root = std::env::temp_dir().join(format!(
            "cliply-installer-replace-test-{}-{unique}",
            std::process::id()
        ));
        fs::create_dir_all(&root).expect("test directory should be created");
        let current = root.join("cliply.exe");
        let replacement = root.join("cliply.new.exe");
        fs::write(&current, b"old").expect("current file should be written");
        fs::write(&replacement, b"new").expect("replacement file should be written");

        let locked_file = OpenOptions::new()
            .read(true)
            .share_mode(FILE_SHARE_READ)
            .open(&current)
            .expect("current file should be locked for the test");
        let release_lock = thread::spawn(move || {
            thread::sleep(Duration::from_millis(600));
            drop(locked_file);
        });

        replace_file_with_retry(&replacement, &current, Duration::from_secs(3))
            .expect("replacement should succeed after the lock is released");
        release_lock.join().expect("lock release should finish");
        assert_eq!(
            fs::read(&current).expect("updated file should exist"),
            b"new"
        );
        assert!(!replacement.exists());

        fs::remove_dir_all(root).expect("test directory should be removed");
    }
}
