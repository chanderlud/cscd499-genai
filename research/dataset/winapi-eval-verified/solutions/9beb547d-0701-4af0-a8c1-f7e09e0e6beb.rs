use std::path::Path;
use std::{iter::once, os::windows::ffi::OsStrExt};
use windows::Win32::Foundation::GENERIC_READ;
use windows::Win32::Storage::FileSystem::{
    CreateFileW, FILE_SHARE_DELETE, FILE_SHARE_READ, GetFileSizeEx, OPEN_EXISTING,
};
use windows::Win32::System::Memory::{
    CreateFileMappingW, FILE_MAP_READ, MapViewOfFile, PAGE_READONLY,
};
use windows::core::Result;

#[derive(Debug)]
pub struct MappedView {
    pub ptr: *const u8,
    pub len: usize,
}

pub fn map_ro(
    path: &Path,
) -> Result<(
    windows::Win32::Foundation::HANDLE,
    windows::Win32::Foundation::HANDLE,
    MappedView,
)> {
    // Convert path to wide string
    let wide_path = wide_null(path)?;

    // Open file with read-only access
    let file_handle = unsafe {
        CreateFileW(
            windows::core::PCWSTR(wide_path.as_ptr()),
            GENERIC_READ.0,
            FILE_SHARE_READ | FILE_SHARE_DELETE,
            None,
            OPEN_EXISTING,
            windows::Win32::Storage::FileSystem::FILE_FLAGS_AND_ATTRIBUTES(0),
            None,
        )?
    };

    // Get file size
    let mut file_size = 0i64;
    unsafe {
        GetFileSizeEx(file_handle, &mut file_size)?;
    }

    // Create file mapping object
    let mapping_handle =
        unsafe { CreateFileMappingW(file_handle, None, PAGE_READONLY, 0, 0, None)? };

    // Map the entire file into memory
    let view_ptr = unsafe {
        let result = MapViewOfFile(mapping_handle, FILE_MAP_READ, 0, 0, file_size as usize);
        if result.Value.is_null() {
            return Err(windows::core::Error::from_thread());
        }
        result
    };

    // Convert to *const u8
    let ptr = view_ptr.Value as *const u8;

    let view = MappedView {
        ptr,
        len: file_size as usize,
    };

    Ok((file_handle, mapping_handle, view))
}

fn wide_null(s: &Path) -> Result<Vec<u16>> {
    let wide: Vec<u16> = s.as_os_str().encode_wide().chain(once(0)).collect();
    Ok(wide)
}
