use crate::error::CliplyError;
use std::ffi::OsStr;
use std::iter::once;
use std::mem::size_of;
use std::os::windows::ffi::OsStrExt;
use std::ptr::null_mut;
use windows_sys::Win32::Foundation::{ERROR_FILE_NOT_FOUND, ERROR_SUCCESS};
use windows_sys::Win32::System::Registry::{
    RegCloseKey, RegDeleteValueW, RegOpenKeyExW, RegSetValueExW, HKEY, HKEY_CURRENT_USER,
    KEY_SET_VALUE, REG_SZ,
};

const RUN_KEY: &str = "Software\\Microsoft\\Windows\\CurrentVersion\\Run";
const RUN_VALUE_NAME: &str = "Cliply";

pub fn set_launch_at_startup(enabled: bool) -> Result<(), CliplyError> {
    let mut key: HKEY = null_mut();
    let open_result = unsafe {
        RegOpenKeyExW(
            HKEY_CURRENT_USER,
            wide_null(RUN_KEY).as_ptr(),
            0,
            KEY_SET_VALUE,
            &mut key,
        )
    };

    if open_result != ERROR_SUCCESS {
        return Err(CliplyError::PlatformUnavailable(format!(
            "failed to open startup registry key: {open_result}"
        )));
    }

    let result = if enabled {
        set_startup_value(key)
    } else {
        delete_startup_value(key)
    };

    unsafe {
        RegCloseKey(key);
    }

    result
}

fn set_startup_value(key: HKEY) -> Result<(), CliplyError> {
    let executable = std::env::current_exe()?;
    let command = format!("\"{}\"", executable.display());
    let value_name = wide_null(RUN_VALUE_NAME);
    let value = wide_null(&command);
    let byte_len = value.len() * size_of::<u16>();
    let result = unsafe {
        RegSetValueExW(
            key,
            value_name.as_ptr(),
            0,
            REG_SZ,
            value.as_ptr().cast(),
            byte_len as u32,
        )
    };

    if result != ERROR_SUCCESS {
        return Err(CliplyError::PlatformUnavailable(format!(
            "failed to set startup registry value: {result}"
        )));
    }

    Ok(())
}

fn delete_startup_value(key: HKEY) -> Result<(), CliplyError> {
    let value_name = wide_null(RUN_VALUE_NAME);
    let result = unsafe { RegDeleteValueW(key, value_name.as_ptr()) };

    if result != ERROR_SUCCESS && result != ERROR_FILE_NOT_FOUND {
        return Err(CliplyError::PlatformUnavailable(format!(
            "failed to delete startup registry value: {result}"
        )));
    }

    Ok(())
}

fn wide_null(value: &str) -> Vec<u16> {
    OsStr::new(value).encode_wide().chain(once(0)).collect()
}
