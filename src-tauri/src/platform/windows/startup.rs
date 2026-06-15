use crate::error::CliplyError;
use std::ffi::{OsStr, OsString};
use std::iter::once;
use std::mem::size_of;
use std::os::windows::ffi::{OsStrExt, OsStringExt};
use std::path::PathBuf;
use std::ptr::null_mut;
use windows_sys::Win32::Foundation::{ERROR_FILE_NOT_FOUND, ERROR_SUCCESS};
use windows_sys::Win32::System::Registry::{
    RegCloseKey, RegCreateKeyExW, RegDeleteValueW, RegOpenKeyExW, RegQueryValueExW, RegSetValueExW,
    HKEY, HKEY_CURRENT_USER, KEY_QUERY_VALUE, KEY_SET_VALUE, REG_OPTION_NON_VOLATILE, REG_SZ,
};

const RUN_KEY: &str = "Software\\Microsoft\\Windows\\CurrentVersion\\Run";
const RUN_VALUE_NAME: &str = "Cliply";
const PRODUCT_EXE: &str = "cliply.exe";

pub fn set_launch_at_startup(enabled: bool, start_minimized: bool) -> Result<(), CliplyError> {
    let key = if enabled {
        open_or_create_startup_key()?
    } else {
        let Some(key) = open_startup_key(KEY_SET_VALUE)? else {
            return Ok(());
        };
        key
    };

    let result = if enabled {
        set_startup_value(key.raw, start_minimized)
    } else {
        delete_startup_value(key.raw)
    };

    result
}

pub fn read_launch_at_startup() -> Result<Option<bool>, CliplyError> {
    let Some(key) = open_startup_key(KEY_QUERY_VALUE)? else {
        return Ok(None);
    };

    read_startup_value(key.raw).map(|value| {
        value.map(|command| {
            command.split_whitespace().any(|part| {
                part.eq_ignore_ascii_case("--minimized")
                    || part.eq_ignore_ascii_case("--start-minimized")
            })
        })
    })
}

fn open_or_create_startup_key() -> Result<StartupKey, CliplyError> {
    let mut key: HKEY = null_mut();
    let result = unsafe {
        RegCreateKeyExW(
            HKEY_CURRENT_USER,
            wide_null(RUN_KEY).as_ptr(),
            0,
            std::ptr::null_mut(),
            REG_OPTION_NON_VOLATILE,
            KEY_SET_VALUE,
            std::ptr::null(),
            &mut key,
            std::ptr::null_mut(),
        )
    };

    if result != ERROR_SUCCESS {
        return Err(CliplyError::PlatformUnavailable(format!(
            "failed to create startup registry key: {result}"
        )));
    }

    Ok(StartupKey { raw: key })
}

fn open_startup_key(access: u32) -> Result<Option<StartupKey>, CliplyError> {
    let mut key: HKEY = null_mut();
    let result = unsafe {
        RegOpenKeyExW(
            HKEY_CURRENT_USER,
            wide_null(RUN_KEY).as_ptr(),
            0,
            access,
            &mut key,
        )
    };

    if result == ERROR_FILE_NOT_FOUND {
        return Ok(None);
    }
    if result != ERROR_SUCCESS {
        return Err(CliplyError::PlatformUnavailable(format!(
            "failed to open startup registry key: {result}"
        )));
    }

    Ok(Some(StartupKey { raw: key }))
}

fn set_startup_value(key: HKEY, start_minimized: bool) -> Result<(), CliplyError> {
    let command = startup_command(start_minimized)?;
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

fn read_startup_value(key: HKEY) -> Result<Option<String>, CliplyError> {
    let value_name = wide_null(RUN_VALUE_NAME);
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

    if size_result == ERROR_FILE_NOT_FOUND {
        return Ok(None);
    }
    if size_result != ERROR_SUCCESS {
        return Err(CliplyError::PlatformUnavailable(format!(
            "failed to read startup registry value size: {size_result}"
        )));
    }
    if value_type != REG_SZ || byte_len < 2 {
        return Ok(None);
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

    if query_result != ERROR_SUCCESS {
        return Err(CliplyError::PlatformUnavailable(format!(
            "failed to read startup registry value: {query_result}"
        )));
    }

    Ok(Some(from_wide_null(&buffer)))
}

fn startup_command(start_minimized: bool) -> Result<String, CliplyError> {
    let executable = startup_executable_path()?;
    let minimized_arg = if start_minimized { " --minimized" } else { "" };
    Ok(format!("\"{}\"{}", executable.display(), minimized_arg))
}

fn startup_executable_path() -> Result<PathBuf, CliplyError> {
    if let Some(install_dir) = super::read_install_dir_from_registry() {
        let installed_exe = PathBuf::from(install_dir).join(PRODUCT_EXE);
        if installed_exe.is_file() {
            return Ok(installed_exe);
        }
    }

    std::env::current_exe().map_err(CliplyError::Filesystem)
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

struct StartupKey {
    raw: HKEY,
}

impl Drop for StartupKey {
    fn drop(&mut self) {
        unsafe {
            RegCloseKey(self.raw);
        }
    }
}

fn wide_null(value: &str) -> Vec<u16> {
    OsStr::new(value).encode_wide().chain(once(0)).collect()
}

fn from_wide_null(value: &[u16]) -> String {
    let len = value.iter().position(|c| *c == 0).unwrap_or(value.len());
    OsString::from_wide(&value[..len])
        .to_string_lossy()
        .to_string()
}
