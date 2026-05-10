pub mod clipboard_listener;
pub mod clipboard_reader;
pub mod clipboard_writer;
pub mod foreground_window;
pub mod paste_simulator;
pub mod startup;

use crate::error::CliplyError;
use std::ffi::{OsStr, OsString};
use std::os::windows::ffi::{OsStrExt, OsStringExt};
use std::ptr::null_mut;
use windows_sys::Win32::Foundation::ERROR_SUCCESS;
use windows_sys::Win32::Foundation::HWND;
use windows_sys::Win32::System::Registry::{
    RegCloseKey, RegOpenKeyExW, RegQueryValueExW, HKEY, HKEY_CURRENT_USER, HKEY_LOCAL_MACHINE,
    KEY_READ, KEY_WOW64_64KEY, REG_SZ,
};
use windows_sys::Win32::UI::Shell::ShellExecuteW;
use windows_sys::Win32::UI::WindowsAndMessaging::SW_SHOWNORMAL;

const PRODUCT_REG_KEY: &str = r"Software\cliply\Cliply";
const PRODUCT_UNINSTALL_KEY: &str = r"Software\Microsoft\Windows\CurrentVersion\Uninstall\Cliply";

pub fn read_install_dir_from_registry() -> Option<String> {
    [
        (
            HKEY_LOCAL_MACHINE,
            PRODUCT_REG_KEY,
            "InstallDir",
            KEY_READ | KEY_WOW64_64KEY,
        ),
        (
            HKEY_LOCAL_MACHINE,
            PRODUCT_REG_KEY,
            "",
            KEY_READ | KEY_WOW64_64KEY,
        ),
        (
            HKEY_LOCAL_MACHINE,
            PRODUCT_UNINSTALL_KEY,
            "InstallLocation",
            KEY_READ | KEY_WOW64_64KEY,
        ),
        (HKEY_CURRENT_USER, PRODUCT_REG_KEY, "InstallDir", KEY_READ),
        (HKEY_CURRENT_USER, PRODUCT_REG_KEY, "", KEY_READ),
        (
            HKEY_CURRENT_USER,
            PRODUCT_UNINSTALL_KEY,
            "InstallLocation",
            KEY_READ,
        ),
    ]
    .into_iter()
    .find_map(|(root, key_path, value_name, access)| {
        read_registry_string(root, key_path, value_name, access)
            .filter(|value| !value.trim().is_empty())
    })
}

pub fn open_url(url: &str) -> Result<(), CliplyError> {
    let operation = wide_null("open");
    let target = wide_null(url);
    let result = unsafe {
        ShellExecuteW(
            std::ptr::null_mut() as HWND,
            operation.as_ptr(),
            target.as_ptr(),
            std::ptr::null(),
            std::ptr::null(),
            SW_SHOWNORMAL,
        )
    } as isize;

    if result > 32 {
        Ok(())
    } else {
        Err(CliplyError::PlatformUnavailable(format!(
            "failed to open url with ShellExecuteW: {result}"
        )))
    }
}

fn read_registry_string(
    root: HKEY,
    key_path: &str,
    value_name: &str,
    access: u32,
) -> Option<String> {
    let mut key: HKEY = null_mut();
    let open_result =
        unsafe { RegOpenKeyExW(root, wide_null(key_path).as_ptr(), 0, access, &mut key) };
    if open_result != ERROR_SUCCESS {
        return None;
    }

    let value_name = wide_null(value_name);
    let mut value_type = 0u32;
    let mut byte_len = 0u32;
    let size_result = unsafe {
        RegQueryValueExW(
            key,
            value_name.as_ptr(),
            std::ptr::null_mut(),
            &mut value_type,
            std::ptr::null_mut(),
            &mut byte_len,
        )
    };
    if size_result != ERROR_SUCCESS || value_type != REG_SZ || byte_len < 2 {
        unsafe {
            RegCloseKey(key);
        }
        return None;
    }

    let mut buffer = vec![0u16; (byte_len as usize + 1) / 2];
    let query_result = unsafe {
        RegQueryValueExW(
            key,
            value_name.as_ptr(),
            std::ptr::null_mut(),
            &mut value_type,
            buffer.as_mut_ptr().cast(),
            &mut byte_len,
        )
    };
    unsafe {
        RegCloseKey(key);
    }

    if query_result != ERROR_SUCCESS {
        return None;
    }

    let len = buffer
        .iter()
        .position(|value| *value == 0)
        .unwrap_or(buffer.len());
    Some(
        OsString::from_wide(&buffer[..len])
            .to_string_lossy()
            .to_string(),
    )
}

fn wide_null(value: &str) -> Vec<u16> {
    OsStr::new(value).encode_wide().chain(Some(0)).collect()
}
