use crate::error::CliplyError;
use crate::platform::ClipboardWritePayload;
use image::DynamicImage;
use std::path::Path;
use std::ptr::{copy_nonoverlapping, null_mut};
use std::thread;
use std::time::Duration;
use windows_sys::Win32::Foundation::{HANDLE, HGLOBAL, HWND};
use windows_sys::Win32::System::DataExchange::{
    CloseClipboard, EmptyClipboard, OpenClipboard, SetClipboardData,
};
use windows_sys::Win32::System::Memory::{
    GlobalAlloc, GlobalLock, GlobalUnlock, GMEM_MOVEABLE, GMEM_ZEROINIT,
};

use super::clipboard_listener;

const CF_UNICODETEXT: u32 = 13;
const CF_DIB: u32 = 8;
const BI_RGB: u32 = 0;

#[link(name = "kernel32")]
extern "system" {
    fn GlobalFree(hmem: HGLOBAL) -> HGLOBAL;
}

pub fn write_payload(
    payload: ClipboardWritePayload,
    owner_window: Option<isize>,
) -> Result<(), CliplyError> {
    clipboard_listener::suppress_clipboard_events_for(Duration::from_millis(700));

    if let Some(image_path) = payload.image_path {
        return write_image_file(&image_path, owner_window);
    }

    let text = payload.text.or(payload.html).ok_or_else(|| {
        CliplyError::PlatformUnavailable("clipboard payload has no writable content".into())
    })?;

    write_unicode_text(&text, owner_window)
}

fn write_unicode_text(value: &str, owner_window: Option<isize>) -> Result<(), CliplyError> {
    let _clipboard = ClipboardGuard::open_with_retry(owner_window)?;
    let mut encoded = value.encode_utf16().collect::<Vec<_>>();
    encoded.push(0);

    let byte_len = encoded.len() * std::mem::size_of::<u16>();
    let handle = allocate_clipboard_bytes(encoded.as_ptr() as *const u8, byte_len)?;

    let emptied = unsafe { EmptyClipboard() } != 0;
    if !emptied {
        unsafe {
            GlobalFree(handle);
        }
        return Err(CliplyError::PlatformUnavailable(
            "failed to empty clipboard".into(),
        ));
    }

    let set_handle = unsafe { SetClipboardData(CF_UNICODETEXT, handle as HANDLE) };
    if set_handle.is_null() {
        unsafe {
            GlobalFree(handle);
        }
        return Err(CliplyError::PlatformUnavailable(
            "failed to set clipboard text".into(),
        ));
    }

    Ok(())
}

fn write_image_file(image_path: &str, owner_window: Option<isize>) -> Result<(), CliplyError> {
    let image = image::open(Path::new(image_path))
        .map_err(|error| CliplyError::StorageUnavailable(error.to_string()))?;
    let dib = image_to_dib(&image)?;
    let handle = allocate_clipboard_bytes(dib.as_ptr(), dib.len())?;
    let _clipboard = ClipboardGuard::open_with_retry(owner_window)?;

    let emptied = unsafe { EmptyClipboard() } != 0;
    if !emptied {
        unsafe {
            GlobalFree(handle);
        }
        return Err(CliplyError::PlatformUnavailable(
            "failed to empty clipboard".into(),
        ));
    }

    let set_handle = unsafe { SetClipboardData(CF_DIB, handle as HANDLE) };
    if set_handle.is_null() {
        unsafe {
            GlobalFree(handle);
        }
        return Err(CliplyError::PlatformUnavailable(
            "failed to set clipboard image".into(),
        ));
    }

    Ok(())
}

fn image_to_dib(image: &DynamicImage) -> Result<Vec<u8>, CliplyError> {
    let rgba = image.to_rgba8();
    let width = rgba.width();
    let height = rgba.height();
    let pixel_bytes = width
        .checked_mul(height)
        .and_then(|pixels| pixels.checked_mul(4))
        .ok_or_else(|| CliplyError::PlatformUnavailable("image is too large".into()))?;
    let mut dib = Vec::with_capacity(40 + pixel_bytes as usize);

    dib.extend_from_slice(&40u32.to_le_bytes());
    dib.extend_from_slice(&(width as i32).to_le_bytes());
    dib.extend_from_slice(&(height as i32).to_le_bytes());
    dib.extend_from_slice(&1u16.to_le_bytes());
    dib.extend_from_slice(&32u16.to_le_bytes());
    dib.extend_from_slice(&BI_RGB.to_le_bytes());
    dib.extend_from_slice(&pixel_bytes.to_le_bytes());
    dib.extend_from_slice(&0i32.to_le_bytes());
    dib.extend_from_slice(&0i32.to_le_bytes());
    dib.extend_from_slice(&0u32.to_le_bytes());
    dib.extend_from_slice(&0u32.to_le_bytes());

    for y in (0..height).rev() {
        for x in 0..width {
            let pixel = rgba.get_pixel(x, y);
            let [red, green, blue, alpha] = pixel.0;
            dib.extend_from_slice(&[blue, green, red, alpha]);
        }
    }

    Ok(dib)
}

fn allocate_clipboard_bytes(source: *const u8, byte_len: usize) -> Result<HGLOBAL, CliplyError> {
    let handle = unsafe { GlobalAlloc(GMEM_MOVEABLE | GMEM_ZEROINIT, byte_len) };
    if handle.is_null() {
        return Err(CliplyError::PlatformUnavailable(
            "failed to allocate clipboard memory".into(),
        ));
    }

    let locked = unsafe { GlobalLock(handle) } as *mut u8;
    if locked.is_null() {
        unsafe {
            GlobalFree(handle);
        }
        return Err(CliplyError::PlatformUnavailable(
            "failed to lock clipboard memory".into(),
        ));
    }

    unsafe {
        copy_nonoverlapping(source, locked, byte_len);
        GlobalUnlock(handle);
    }

    Ok(handle)
}

struct ClipboardGuard;

impl ClipboardGuard {
    fn open_with_retry(owner_window: Option<isize>) -> Result<Self, CliplyError> {
        let owner = owner_window
            .map(|handle| handle as HWND)
            .unwrap_or_else(null_mut);
        let retry_delays = [
            Duration::from_millis(15),
            Duration::from_millis(30),
            Duration::from_millis(60),
            Duration::from_millis(100),
            Duration::from_millis(160),
            Duration::from_millis(240),
        ];

        for delay in retry_delays {
            let opened = unsafe { OpenClipboard(owner) } != 0;
            if opened {
                return Ok(Self);
            }

            thread::sleep(delay);
        }

        let opened = unsafe { OpenClipboard(owner) } != 0;
        if opened {
            return Ok(Self);
        }

        Err(CliplyError::PlatformUnavailable(
            "windows clipboard is currently unavailable after retry".into(),
        ))
    }
}

impl Drop for ClipboardGuard {
    fn drop(&mut self) {
        unsafe {
            CloseClipboard();
        }
    }
}
