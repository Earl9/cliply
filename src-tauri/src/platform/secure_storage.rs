use crate::error::CliplyError;

#[cfg(target_os = "windows")]
pub fn protect_bytes(data: &[u8]) -> Result<Vec<u8>, CliplyError> {
    windows_secure_storage::protect_bytes(data)
}

#[cfg(target_os = "windows")]
pub fn unprotect_bytes(data: &[u8]) -> Result<Vec<u8>, CliplyError> {
    windows_secure_storage::unprotect_bytes(data)
}

#[cfg(not(target_os = "windows"))]
pub fn protect_bytes(data: &[u8]) -> Result<Vec<u8>, CliplyError> {
    Ok(data.to_vec())
}

#[cfg(not(target_os = "windows"))]
pub fn unprotect_bytes(data: &[u8]) -> Result<Vec<u8>, CliplyError> {
    Ok(data.to_vec())
}

#[cfg(target_os = "windows")]
mod windows_secure_storage {
    use super::CliplyError;
    use std::ffi::c_void;
    use std::ptr::{null, null_mut};

    const CRYPTPROTECT_UI_FORBIDDEN: u32 = 0x1;

    #[repr(C)]
    struct DataBlob {
        cb_data: u32,
        pb_data: *mut u8,
    }

    #[link(name = "crypt32")]
    extern "system" {
        fn CryptProtectData(
            data_in: *mut DataBlob,
            data_description: *const u16,
            optional_entropy: *mut DataBlob,
            reserved: *mut c_void,
            prompt_struct: *const c_void,
            flags: u32,
            data_out: *mut DataBlob,
        ) -> i32;

        fn CryptUnprotectData(
            data_in: *mut DataBlob,
            data_description: *mut *mut u16,
            optional_entropy: *mut DataBlob,
            reserved: *mut c_void,
            prompt_struct: *const c_void,
            flags: u32,
            data_out: *mut DataBlob,
        ) -> i32;
    }

    #[link(name = "kernel32")]
    extern "system" {
        fn LocalFree(memory: *mut c_void) -> *mut c_void;
    }

    pub fn protect_bytes(data: &[u8]) -> Result<Vec<u8>, CliplyError> {
        let mut input = DataBlob {
            cb_data: data.len() as u32,
            pb_data: data.as_ptr() as *mut u8,
        };
        let mut output = DataBlob {
            cb_data: 0,
            pb_data: null_mut(),
        };

        let ok = unsafe {
            CryptProtectData(
                &mut input,
                null(),
                null_mut(),
                null_mut(),
                null(),
                CRYPTPROTECT_UI_FORBIDDEN,
                &mut output,
            )
        };
        if ok == 0 {
            return Err(CliplyError::PlatformUnavailable(format!(
                "Windows DPAPI protect failed: {}",
                std::io::Error::last_os_error()
            )));
        }

        copy_and_free_blob(output)
    }

    pub fn unprotect_bytes(data: &[u8]) -> Result<Vec<u8>, CliplyError> {
        let mut input = DataBlob {
            cb_data: data.len() as u32,
            pb_data: data.as_ptr() as *mut u8,
        };
        let mut output = DataBlob {
            cb_data: 0,
            pb_data: null_mut(),
        };

        let ok = unsafe {
            CryptUnprotectData(
                &mut input,
                null_mut(),
                null_mut(),
                null_mut(),
                null(),
                CRYPTPROTECT_UI_FORBIDDEN,
                &mut output,
            )
        };
        if ok == 0 {
            return Err(CliplyError::PlatformUnavailable(format!(
                "Windows DPAPI unprotect failed: {}",
                std::io::Error::last_os_error()
            )));
        }

        copy_and_free_blob(output)
    }

    fn copy_and_free_blob(blob: DataBlob) -> Result<Vec<u8>, CliplyError> {
        if blob.pb_data.is_null() {
            return Ok(Vec::new());
        }

        let bytes =
            unsafe { std::slice::from_raw_parts(blob.pb_data, blob.cb_data as usize).to_vec() };
        unsafe {
            let _ = LocalFree(blob.pb_data as _);
        }
        Ok(bytes)
    }
}
