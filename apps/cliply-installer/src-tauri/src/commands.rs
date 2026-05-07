use tauri::Emitter;

use crate::installer::{
    self, InstallDetection, InstallOptions, InstallOutcome, InstallerMode, UninstallOptions,
    UninstallOutcome,
};

#[tauri::command]
pub fn detect_mode() -> InstallerMode {
    installer::detect_mode()
}

#[tauri::command]
pub fn detect_installation() -> Result<InstallDetection, String> {
    installer::detect_installation().map_err(|error| error.to_string())
}

#[tauri::command]
pub fn browse_install_dir(current_dir: String) -> Result<Option<String>, String> {
    crate::platform::browse_install_dir(&current_dir).map_err(|error| error.to_string())
}

#[tauri::command]
pub fn run_install(
    app: tauri::AppHandle,
    options: InstallOptions,
) -> Result<InstallOutcome, String> {
    installer::install(options, |event| {
        let _ = app.emit("installer-progress", event);
    })
    .map_err(|error| error.to_string())
}

#[tauri::command]
pub fn run_uninstall(
    app: tauri::AppHandle,
    options: UninstallOptions,
) -> Result<UninstallOutcome, String> {
    installer::uninstall(options, |event| {
        let _ = app.emit("installer-progress", event);
    })
    .map_err(|error| error.to_string())
}

#[tauri::command]
pub fn launch_cliply(install_dir: String) -> Result<(), String> {
    installer::launch_cliply(install_dir).map_err(|error| error.to_string())
}
