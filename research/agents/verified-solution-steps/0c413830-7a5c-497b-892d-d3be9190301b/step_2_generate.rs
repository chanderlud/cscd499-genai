#[allow(unused_imports)]
use windows::core::{Result, Error};
use std::path::{Path, PathBuf};
use std::ffi::OsStr;

fn wide_null(s: &OsStr) -> Vec<u16> {
    use std::{iter::once, os::windows::ffi::OsStrExt};
    s.encode_wide().chain(once(0)).collect()
}

pub fn full_path(path: &Path) -> Result<PathBuf> {
    use windows::Win32::Storage::FileSystem::GetFullPathNameW;

    let wide_path = wide_null(path.as_os_str());

    // First call to get required buffer size
    let len = unsafe {
        GetFullPathNameW(
            windows::core::PCWSTR(wide_path.as_ptr()),
            None,
            None,
        )
    };

    if len == 0 {
        return Err(Error::from_thread());
    }

    // Allocate buffer with required size
    let mut buffer = vec![0u16; len as usize];

    let result = unsafe {
        GetFullPathNameW(
            windows::core::PCWSTR(wide_path.as_ptr()),
            Some(&mut buffer),
            None,
        )
    };

    if result == 0 {
        return Err(Error::from_thread());
    }

    // Convert result to PathBuf
    let path_str = String::from_utf16_lossy(&buffer[..result as usize]);
    Ok(PathBuf::from(path_str))
}