use crate::error::CliplyError;
use crate::models::clipboard_item::ClipboardItemType;
use crate::platform::{ClipboardFormatSnapshot, ClipboardSnapshot};
use crate::services::content_detector;
use std::ffi::c_void;
use std::slice;
use windows_sys::Win32::Foundation::HGLOBAL;
use windows_sys::Win32::System::DataExchange::{
    CloseClipboard, EnumClipboardFormats, GetClipboardData, GetClipboardFormatNameW,
    IsClipboardFormatAvailable, OpenClipboard,
};
use windows_sys::Win32::System::Memory::{GlobalLock, GlobalSize, GlobalUnlock};

const CF_TEXT: u32 = 1;
const CF_BITMAP: u32 = 2;
const CF_DIB: u32 = 8;
const CF_DIBV5: u32 = 17;
const CF_UNICODETEXT: u32 = 13;

pub fn read_current_snapshot() -> Result<Option<ClipboardSnapshot>, CliplyError> {
    let _clipboard = ClipboardGuard::open()?;
    let formats = read_available_formats();
    let text = read_unicode_text()?;

    if let Some(text) = text {
        if text.trim().is_empty() {
            return Ok(None);
        }

        let primary_type = content_detector::detect_text_type(&text);
        return Ok(Some(ClipboardSnapshot {
            primary_type,
            text: Some(text),
            html: None,
            image: None,
            formats,
            source_app: Some("Windows Clipboard".into()),
            source_window: None,
        }));
    }

    if is_format_available(CF_BITMAP)
        || is_format_available(CF_DIB)
        || is_format_available(CF_DIBV5)
    {
        return Ok(Some(ClipboardSnapshot {
            primary_type: ClipboardItemType::Image,
            text: None,
            html: None,
            image: None,
            formats,
            source_app: Some("Windows Clipboard".into()),
            source_window: None,
        }));
    }

    Ok(None)
}

fn read_unicode_text() -> Result<Option<String>, CliplyError> {
    if !is_format_available(CF_UNICODETEXT) {
        return Ok(None);
    }

    let handle = unsafe { GetClipboardData(CF_UNICODETEXT) };
    if handle.is_null() {
        return Ok(None);
    }

    let global = handle as HGLOBAL;
    let locked = unsafe { GlobalLock(global) } as *const u16;
    if locked.is_null() {
        return Ok(None);
    }

    let byte_len = unsafe { GlobalSize(global) };
    let max_units = byte_len / std::mem::size_of::<u16>();
    let units = unsafe { slice::from_raw_parts(locked, max_units) };
    let len = units
        .iter()
        .position(|unit| *unit == 0)
        .unwrap_or(max_units);
    let value = String::from_utf16_lossy(&units[..len]);

    unsafe {
        GlobalUnlock(global);
    }

    Ok(Some(value))
}

fn read_available_formats() -> Vec<ClipboardFormatSnapshot> {
    let mut formats = Vec::new();
    let mut current = 0;

    loop {
        current = unsafe { EnumClipboardFormats(current) };
        if current == 0 {
            break;
        }

        formats.push(ClipboardFormatSnapshot {
            format_name: format_name(current),
            mime_type: mime_type(current),
            data_kind: data_kind(current).into(),
        });
    }

    formats
}

fn format_name(format: u32) -> String {
    match format {
        CF_TEXT => "CF_TEXT".into(),
        CF_BITMAP => "CF_BITMAP".into(),
        CF_DIB => "CF_DIB".into(),
        CF_DIBV5 => "CF_DIBV5".into(),
        CF_UNICODETEXT => "CF_UNICODETEXT".into(),
        _ => registered_format_name(format).unwrap_or_else(|| format!("Windows format {format}")),
    }
}

fn registered_format_name(format: u32) -> Option<String> {
    let mut buffer = [0u16; 128];
    let len = unsafe { GetClipboardFormatNameW(format, buffer.as_mut_ptr(), buffer.len() as i32) };
    if len <= 0 {
        return None;
    }

    Some(String::from_utf16_lossy(&buffer[..len as usize]))
}

fn mime_type(format: u32) -> Option<String> {
    match format {
        CF_TEXT | CF_UNICODETEXT => Some("text/plain".into()),
        CF_BITMAP | CF_DIB | CF_DIBV5 => Some("image/bmp".into()),
        _ => None,
    }
}

fn data_kind(format: u32) -> &'static str {
    match format {
        CF_TEXT | CF_UNICODETEXT => "text",
        CF_BITMAP | CF_DIB | CF_DIBV5 => "image_file",
        _ => "external_ref",
    }
}

fn is_format_available(format: u32) -> bool {
    (unsafe { IsClipboardFormatAvailable(format) }) != 0
}

struct ClipboardGuard;

impl ClipboardGuard {
    fn open() -> Result<Self, CliplyError> {
        let opened = unsafe { OpenClipboard(std::ptr::null_mut::<c_void>()) } != 0;
        if !opened {
            return Err(CliplyError::PlatformUnavailable(
                "windows clipboard is currently unavailable".into(),
            ));
        }

        Ok(Self)
    }
}

impl Drop for ClipboardGuard {
    fn drop(&mut self) {
        unsafe {
            CloseClipboard();
        }
    }
}
