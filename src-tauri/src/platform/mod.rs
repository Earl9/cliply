pub mod linux;
pub mod macos;
pub mod secure_storage;
pub mod theme_adapter;
pub mod windows;

use crate::error::CliplyError;
use crate::models::clipboard_item::ClipboardItemType;
use tauri::AppHandle;

#[derive(Debug, Clone)]
pub struct ClipboardSnapshot {
    pub primary_type: ClipboardItemType,
    pub text: Option<String>,
    #[allow(dead_code)]
    pub html: Option<String>,
    pub image: Option<ImageSnapshot>,
    pub formats: Vec<ClipboardFormatSnapshot>,
    pub source_app: Option<String>,
    pub source_window: Option<String>,
}

#[derive(Debug, Clone)]
pub struct ImageSnapshot {
    pub width: u32,
    pub height: u32,
    pub bytes: Vec<u8>,
    pub mime_type: String,
    pub extension: String,
}

#[derive(Debug, Clone)]
pub struct ClipboardFormatSnapshot {
    pub format_name: String,
    pub mime_type: Option<String>,
    pub data_kind: String,
}

#[derive(Debug, Clone)]
pub struct ClipboardWritePayload {
    pub text: Option<String>,
    pub html: Option<String>,
    pub image_path: Option<String>,
}

#[derive(Debug, Clone)]
pub struct ForegroundAppInfo {
    pub app_name: String,
    pub window_title: Option<String>,
}

#[allow(dead_code)]
pub trait ClipboardPlatform {
    fn start_listening(&self) -> Result<(), CliplyError>;
    fn stop_listening(&self) -> Result<(), CliplyError>;
    fn read_clipboard(&self) -> Result<ClipboardSnapshot, CliplyError>;
    fn write_clipboard(&self, item: ClipboardWritePayload) -> Result<(), CliplyError>;
    fn paste_to_foreground(&self) -> Result<(), CliplyError>;
    fn get_foreground_app(&self) -> Result<Option<ForegroundAppInfo>, CliplyError>;
}

pub fn start_clipboard_listener(app: AppHandle) -> Result<(), CliplyError> {
    #[cfg(target_os = "windows")]
    {
        return windows::clipboard_listener::start_listener(app);
    }

    #[cfg(not(target_os = "windows"))]
    {
        let _ = app;
        Ok(())
    }
}

pub fn stop_clipboard_listener() -> Result<(), CliplyError> {
    #[cfg(target_os = "windows")]
    {
        return windows::clipboard_listener::stop_listener();
    }

    #[cfg(not(target_os = "windows"))]
    {
        Ok(())
    }
}

pub fn read_current_clipboard() -> Result<Option<ClipboardSnapshot>, CliplyError> {
    #[cfg(target_os = "windows")]
    {
        return windows::clipboard_reader::read_current_snapshot();
    }

    #[cfg(not(target_os = "windows"))]
    {
        Ok(None)
    }
}

pub fn get_foreground_app() -> Option<ForegroundAppInfo> {
    #[cfg(target_os = "windows")]
    {
        return windows::foreground_window::current_foreground_app();
    }

    #[cfg(not(target_os = "windows"))]
    {
        None
    }
}

pub fn write_clipboard_payload(
    payload: ClipboardWritePayload,
    owner_window: Option<isize>,
) -> Result<(), CliplyError> {
    #[cfg(target_os = "windows")]
    {
        return windows::clipboard_writer::write_payload(payload, owner_window);
    }

    #[cfg(not(target_os = "windows"))]
    {
        let _ = payload;
        let _ = owner_window;
        Ok(())
    }
}

pub fn paste_to_foreground() -> Result<(), CliplyError> {
    #[cfg(target_os = "windows")]
    {
        return windows::paste_simulator::simulate_ctrl_v();
    }

    #[cfg(not(target_os = "windows"))]
    {
        Ok(())
    }
}

pub fn set_launch_at_startup(enabled: bool, start_minimized: bool) -> Result<(), CliplyError> {
    #[cfg(target_os = "windows")]
    {
        return windows::startup::set_launch_at_startup(enabled, start_minimized);
    }

    #[cfg(not(target_os = "windows"))]
    {
        let _ = enabled;
        let _ = start_minimized;
        Ok(())
    }
}
