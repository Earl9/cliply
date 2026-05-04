pub mod linux;
pub mod macos;
pub mod windows;

use crate::error::CliplyError;
use crate::models::clipboard_item::ClipboardItemType;
use tauri::AppHandle;

#[derive(Debug, Clone)]
pub struct ClipboardSnapshot {
    pub primary_type: ClipboardItemType,
    pub text: Option<String>,
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
