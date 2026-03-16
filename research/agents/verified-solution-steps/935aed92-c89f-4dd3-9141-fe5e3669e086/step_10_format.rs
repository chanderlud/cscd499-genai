use std::io;
use std::path::Path;
use std::slice;

use windows::core::PCWSTR;
use windows::Win32::Foundation::{CloseHandle, ERROR_FILE_NOT_FOUND, ERROR_PATH_NOT_FOUND, HANDLE};
use windows::Win32::Storage::FileSystem::{
    CreateFileW, GetFileSizeEx, FILE_ATTRIBUTE_NORMAL, FILE_GENERIC_READ, FILE_SHARE_READ,
    OPEN_EXISTING,
};
use windows::Win32::System::Memory::{
    CreateFileMappingW, MapViewOfFile, UnmapViewOfFile, FILE_MAP_READ, MEMORY_MAPPED_VIEW_ADDRESS,
    PAGE_READONLY,
};

fn wide_null(s: &std::ffi::OsStr) -> Vec<u16> {
    use std::iter::once;
    use std::os::windows::ffi::OsStrExt;
    s.encode_wide().chain(once(0)).collect()
}

struct MappedFile {
    file_handle: HANDLE,
    mapping_handle: HANDLE,
    view: MEMORY_MAPPED_VIEW_ADDRESS,
    file_size: usize,
}

impl MappedFile {
    fn new(path: &Path) -> io::Result<Self> {
        let wide_path = wide_null(path.as_os_str());

        // SAFETY: CreateFileW is called with valid parameters
        let file_handle = unsafe {
            CreateFileW(
                PCWSTR(wide_path.as_ptr()),
                FILE_GENERIC_READ.0,
                FILE_SHARE_READ,
                Some(std::ptr::null()),
                OPEN_EXISTING,
                FILE_ATTRIBUTE_NORMAL,
                Some(HANDLE::default()),
            )
        }
        .map_err(|e| {
            // Extract just the error code from HRESULT (lower 16 bits)
            let error_code = e.code().0 as u32 & 0xFFFF;

            // Check for file not found errors
            if error_code == ERROR_FILE_NOT_FOUND.0 || error_code == ERROR_PATH_NOT_FOUND.0 {
                return io::Error::new(io::ErrorKind::NotFound, "File not found");
            }

            io::Error::new(io::ErrorKind::Other, e.to_string())
        })?;

        // Get file size
        let mut file_size_i64 = 0i64;
        // SAFETY: GetFileSizeEx is called with valid handle and pointer
        unsafe { GetFileSizeEx(file_handle, &mut file_size_i64) }
            .map_err(|e| io::Error::new(io::ErrorKind::Other, e.to_string()))?;

        let file_size = file_size_i64 as usize;

        // Handle empty files specially - can't map zero-length files
        if file_size == 0 {
            return Ok(Self {
                file_handle,
                mapping_handle: HANDLE::default(), // Invalid handle
                view: MEMORY_MAPPED_VIEW_ADDRESS {
                    Value: std::ptr::null_mut(),
                },
                file_size: 0,
            });
        }

        // SAFETY: CreateFileMappingW is called with valid parameters
        let mapping_handle = unsafe {
            CreateFileMappingW(
                file_handle,
                Some(std::ptr::null()),
                PAGE_READONLY,
                0,
                0,
                PCWSTR::null(),
            )
        }
        .map_err(|e| {
            unsafe {
                let _ = CloseHandle(file_handle);
            }
            io::Error::new(io::ErrorKind::Other, e.to_string())
        })?;

        // SAFETY: MapViewOfFile is called with valid parameters
        let view = unsafe { MapViewOfFile(mapping_handle, FILE_MAP_READ, 0, 0, 0) };

        if view.Value.is_null() {
            unsafe {
                let _ = CloseHandle(mapping_handle);
                let _ = CloseHandle(file_handle);
            }
            return Err(io::Error::new(io::ErrorKind::Other, "MapViewOfFile failed"));
        }

        Ok(Self {
            file_handle,
            mapping_handle,
            view,
            file_size,
        })
    }

    fn as_bytes(&self) -> &[u8] {
        if self.file_size == 0 {
            &[]
        } else {
            // SAFETY: view is valid for file_size bytes and we have exclusive access
            unsafe { slice::from_raw_parts(self.view.Value as *const u8, self.file_size) }
        }
    }
}

impl Drop for MappedFile {
    fn drop(&mut self) {
        unsafe {
            if !self.view.Value.is_null() {
                let _ = UnmapViewOfFile(self.view);
            }
            if !self.mapping_handle.is_invalid() {
                let _ = CloseHandle(self.mapping_handle);
            }
            if !self.file_handle.is_invalid() {
                let _ = CloseHandle(self.file_handle);
            }
        }
    }
}

pub fn mmap_count_nonoverlapping(path: &Path, needle: &[u8]) -> io::Result<usize> {
    if needle.is_empty() {
        return Ok(0);
    }

    let mapped = MappedFile::new(path)?;
    let data = mapped.as_bytes();

    let mut count = 0;
    let mut pos = 0;

    while pos + needle.len() <= data.len() {
        if &data[pos..pos + needle.len()] == needle {
            count += 1;
            pos += needle.len();
        } else {
            pos += 1;
        }
    }

    Ok(count)
}