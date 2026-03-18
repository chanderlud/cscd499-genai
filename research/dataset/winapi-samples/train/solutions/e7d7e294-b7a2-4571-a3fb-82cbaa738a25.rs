use std::os::windows::ffi::OsStrExt;
use std::path::Path;
use windows::core::{Error, Result, HRESULT, PCWSTR};
use windows::Win32::Foundation::{
    ERROR_ALREADY_EXISTS, ERROR_FILENAME_EXCED_RANGE, MAX_PATH, WIN32_ERROR,
};
use windows::Win32::Storage::FileSystem::CreateDirectoryW;

const MAX_PATH_USIZE: usize = MAX_PATH as usize;

fn wide_null_fixed(path: &Path) -> Result<[u16; MAX_PATH_USIZE + 1]> {
    let mut buffer = [0u16; MAX_PATH_USIZE + 1];
    let os_str = path.as_os_str();
    let mut i = 0;

    for wide_char in os_str.encode_wide() {
        if i >= MAX_PATH_USIZE {
            return Err(Error::from_hresult(HRESULT::from_win32(
                ERROR_FILENAME_EXCED_RANGE.0,
            )));
        }
        buffer[i] = wide_char;
        i += 1;
    }

    // Ensure null termination
    buffer[i] = 0;
    Ok(buffer)
}

pub fn create_dir_all_no_alloc(path: &Path) -> Result<()> {
    let mut buffer = wide_null_fixed(path)?;

    // Find the root component length (skip drive letter or UNC prefix)
    let mut start_idx = 0;
    if buffer.len() > 2 && buffer[1] == ':' as u16 {
        // Drive letter path (e.g., "C:\")
        start_idx = 3; // Skip "C:\"
    } else if buffer.len() > 2 && buffer[0] == '\\' as u16 && buffer[1] == '\\' as u16 {
        // UNC path (e.g., "\\server\share")
        // Find the end of the server name
        let mut i = 2;
        while i < buffer.len() && buffer[i] != 0 && buffer[i] != '\\' as u16 {
            i += 1;
        }
        if i < buffer.len() && buffer[i] == '\\' as u16 {
            i += 1;
            // Find the end of the share name
            while i < buffer.len() && buffer[i] != 0 && buffer[i] != '\\' as u16 {
                i += 1;
            }
            start_idx = i;
        }
    }

    // Create directories component by component
    let mut i = start_idx;
    while i < buffer.len() && buffer[i] != 0 {
        // Find next separator or end
        while i < buffer.len()
            && buffer[i] != 0
            && buffer[i] != '\\' as u16
            && buffer[i] != '/' as u16
        {
            i += 1;
        }

        if i < buffer.len() && (buffer[i] == '\\' as u16 || buffer[i] == '/' as u16) {
            // Temporarily null-terminate at this separator
            let saved = buffer[i];
            buffer[i] = 0;

            // Try to create the directory
            let result = unsafe { CreateDirectoryW(PCWSTR(buffer.as_ptr()), None) };

            // Restore the separator
            buffer[i] = saved;

            // Handle the result
            if let Err(e) = result {
                if e.code() != ERROR_ALREADY_EXISTS.to_hresult() {
                    return Err(e);
                }
            }

            i += 1;
        } else {
            // Reached the end - create the final directory
            let result = unsafe { CreateDirectoryW(PCWSTR(buffer.as_ptr()), None) };

            if let Err(e) = result {
                if e.code() != ERROR_ALREADY_EXISTS.to_hresult() {
                    return Err(e);
                }
            }
            break;
        }
    }

    Ok(())
}
