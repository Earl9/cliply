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
const PRODUCT_ICON: &str = "cliply.ico";
const PRODUCT_UNINSTALLER: &str = "uninstall.exe";
const PRODUCT_REG_KEY: &str = r"Software\cliply\Cliply";
const PRODUCT_UNINSTALL_KEY: &str = r"Software\Microsoft\Windows\CurrentVersion\Uninstall\Cliply";
const START_MENU_FOLDER: &str = "Cliply";

#[derive(Debug, Error)]
pub enum InstallError {
    #[error("安装路径不能为空")]
    EmptyInstallDir,
    #[error("安装路径不合法：{0}")]
    InvalidInstallDir(String),
    #[error("无法创建目录 {path}: {source}")]
    CreateDir {
        path: String,
        source: std::io::Error,
    },
    #[error("无法写入文件 {path}: {source}")]
    WriteFile {
        path: String,
        source: std::io::Error,
    },
    #[error("无法替换旧文件 {path}。Cliply 可能仍在运行，或文件被安全软件占用。请退出 Cliply 后重试；如果仍失败，请重启 Windows 再安装。原始错误：{source}")]
    ReplaceLockedFile {
        path: String,
        source: std::io::Error,
    },
    #[error("无法关闭正在运行的 Cliply。请从托盘退出 Cliply 后重试。")]
    StopRunningCliply,
    #[error("无法启动 Cliply: {0}")]
    Launch(std::io::Error),
    #[error("无法读取当前安装器路径: {0}")]
    CurrentExe(std::io::Error),
    #[error("无法解压 Cliply 程序文件: {0}")]
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
        wait_for_process_exit(parent_pid, Duration::from_secs(10));
    }

    on_progress(progress(8, "正在关闭正在运行的 Cliply"));
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
    )?;

    on_progress(progress(72, "正在创建开始菜单快捷方式"));
    platform::create_start_menu_shortcuts(START_MENU_FOLDER, &exe_path, &icon_path)?;

    on_progress(progress(86, "正在应用你的安装选项"));
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
    on_progress(progress(
        100,
        if preserve_user_data {
            "安装完成，用户数据已保留"
        } else {
            "安装完成"
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
    on_progress(progress(10, "正在关闭正在运行的 Cliply"));
    stop_running_cliply()?;

    on_progress(progress(28, "正在移除快捷方式和开机启动"));
    let exe_path = install_dir.join(PRODUCT_EXE);
    let _ = platform::set_start_on_login(PRODUCT_NAME, &exe_path, false);
    let _ = platform::remove_desktop_shortcut();
    let _ = platform::remove_start_menu_shortcuts(START_MENU_FOLDER);

    on_progress(progress(52, "正在删除 Cliply 程序文件"));
    remove_installed_files(&install_dir)?;

    on_progress(progress(72, "正在清理安装信息"));
    platform::remove_install_registry(PRODUCT_REG_KEY, PRODUCT_UNINSTALL_KEY)?;

    if options.remove_user_data {
        on_progress(progress(88, "正在删除本地历史记录与设置"));
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
    let files = [
        PRODUCT_EXE,
        PRODUCT_ICON,
        "Uninstall Cliply.lnk",
        "卸载 Cliply.lnk",
    ];

    for file in files {
        let path = install_dir.join(file);
        if path.exists() {
            remove_file_with_retry(&path)?;
        }
    }

    Ok(())
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

    if path.exists() {
        remove_file_with_retry(path)?;
    }

    rename_file_with_retry(&temp_path, path).map_err(|error| {
        let _ = fs::remove_file(&temp_path);
        error
    })
}

fn stop_running_cliply() -> InstallResult<()> {
    if !is_cliply_running() {
        return Ok(());
    }

    let _ = Command::new("taskkill")
        .args(["/IM", PRODUCT_EXE, "/F", "/T"])
        .creation_flags_no_window()
        .status();

    wait_until_cliply_exits(Duration::from_secs(8))
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

    let mut last_error = None;
    for _ in 0..12 {
        match fs::remove_file(path) {
            Ok(()) => return Ok(()),
            Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(()),
            Err(error) => {
                last_error = Some(error);
                thread::sleep(Duration::from_millis(300));
            }
        }
    }

    Err(InstallError::ReplaceLockedFile {
        path: path.to_string_lossy().to_string(),
        source: last_error
            .unwrap_or_else(|| io::Error::new(io::ErrorKind::Other, "unknown remove error")),
    })
}

fn rename_file_with_retry(from: &Path, to: &Path) -> InstallResult<()> {
    let mut last_error = None;
    for _ in 0..12 {
        match fs::rename(from, to) {
            Ok(()) => return Ok(()),
            Err(error) => {
                last_error = Some(error);
                thread::sleep(Duration::from_millis(300));
            }
        }
    }

    Err(InstallError::ReplaceLockedFile {
        path: to.to_string_lossy().to_string(),
        source: last_error
            .unwrap_or_else(|| io::Error::new(io::ErrorKind::Other, "unknown rename error")),
    })
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
