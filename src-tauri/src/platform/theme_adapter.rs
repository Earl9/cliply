use crate::error::CliplyError;
use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SystemThemeColorInfo {
    pub system_accent: Option<String>,
    pub source: String,
    pub status: String,
    pub message: String,
}

pub fn read_system_theme_colors() -> Result<SystemThemeColorInfo, CliplyError> {
    #[cfg(target_os = "windows")]
    {
        return windows_theme_adapter::read_system_theme_colors();
    }

    #[cfg(not(target_os = "windows"))]
    {
        Ok(SystemThemeColorInfo {
            system_accent: None,
            source: "fallback".to_string(),
            status: "fallback".to_string(),
            message: "当前平台暂未接入系统强调色读取，已使用 Cliply 默认 fallback。".to_string(),
        })
    }
}

#[cfg(target_os = "windows")]
mod windows_theme_adapter {
    use super::SystemThemeColorInfo;
    use crate::error::CliplyError;
    use std::ffi::OsStr;
    use std::os::windows::ffi::OsStrExt;
    use std::ptr::null_mut;
    use windows_sys::Win32::Foundation::{ERROR_FILE_NOT_FOUND, ERROR_SUCCESS};
    use windows_sys::Win32::System::Registry::{
        RegCloseKey, RegGetValueW, RegOpenKeyExW, HKEY, HKEY_CURRENT_USER, KEY_QUERY_VALUE,
        RRF_RT_REG_DWORD,
    };

    const DWM_KEY: &str = r"Software\Microsoft\Windows\DWM";
    pub fn read_system_theme_colors() -> Result<SystemThemeColorInfo, CliplyError> {
        let accent = read_accent_color()?;

        if accent.is_some() {
            return Ok(SystemThemeColorInfo {
                system_accent: accent,
                source: "windows-registry".to_string(),
                status: "ok".to_string(),
                message: "已读取 Windows 强调色。".to_string(),
            });
        }

        Ok(SystemThemeColorInfo {
            system_accent: None,
            source: "fallback".to_string(),
            status: "fallback".to_string(),
            message: "未读取到 Windows 强调色，已使用 Cliply 默认 fallback。".to_string(),
        })
    }

    fn read_accent_color() -> Result<Option<String>, CliplyError> {
        for value_name in ["AccentColor", "ColorizationColor"] {
            match read_registry_dword(HKEY_CURRENT_USER, DWM_KEY, value_name) {
                Ok(value) => return Ok(Some(windows_color_dword_to_hex(value))),
                Err(error) if is_not_found(&error) => continue,
                Err(error) => return Err(error),
            }
        }
        Ok(None)
    }

    fn read_registry_dword(
        root: HKEY,
        key_path: &str,
        value_name: &str,
    ) -> Result<u32, CliplyError> {
        let mut key: HKEY = null_mut();
        let key_path = wide_null(key_path);
        let open_status =
            unsafe { RegOpenKeyExW(root, key_path.as_ptr(), 0, KEY_QUERY_VALUE, &mut key) };
        if open_status != ERROR_SUCCESS {
            return Err(registry_error("open", key_path_error(open_status)));
        }

        let mut value: u32 = 0;
        let mut value_size = std::mem::size_of::<u32>() as u32;
        let value_name = wide_null(value_name);
        let status = unsafe {
            RegGetValueW(
                key,
                null_mut(),
                value_name.as_ptr(),
                RRF_RT_REG_DWORD,
                null_mut(),
                &mut value as *mut u32 as *mut _,
                &mut value_size,
            )
        };
        unsafe {
            RegCloseKey(key);
        }

        if status != ERROR_SUCCESS {
            return Err(registry_error("read", key_path_error(status)));
        }

        Ok(value)
    }

    fn windows_color_dword_to_hex(value: u32) -> String {
        let red = value & 0xFF;
        let green = (value >> 8) & 0xFF;
        let blue = (value >> 16) & 0xFF;
        format!("#{red:02X}{green:02X}{blue:02X}")
    }

    fn wide_null(value: &str) -> Vec<u16> {
        OsStr::new(value)
            .encode_wide()
            .chain(std::iter::once(0))
            .collect()
    }

    fn key_path_error(code: u32) -> String {
        if code == ERROR_FILE_NOT_FOUND {
            return "not found".to_string();
        }
        format!("Windows registry error {code}")
    }

    fn registry_error(action: &str, reason: String) -> CliplyError {
        CliplyError::PlatformUnavailable(format!(
            "Windows theme registry {action} failed: {reason}"
        ))
    }

    fn is_not_found(error: &CliplyError) -> bool {
        error.to_string().to_ascii_lowercase().contains("not found")
    }
}
