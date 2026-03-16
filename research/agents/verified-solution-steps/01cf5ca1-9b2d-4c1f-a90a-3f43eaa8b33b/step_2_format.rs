use std::ffi::OsStr;
#[allow(unused_imports)]
use windows::core::{Error, Result};
use windows::Win32::Foundation::{CloseHandle, E_INVALIDARG, HANDLE, INVALID_HANDLE_VALUE};
use windows::Win32::System::Memory::{
    CreateFileMappingW, MapViewOfFile, OpenFileMappingW, UnmapViewOfFile, FILE_MAP_READ,
    FILE_MAP_WRITE, PAGE_READWRITE,
};

fn wide_null(s: &std::ffi::OsStr) -> Vec<u16> {
    use std::{iter::once, os::windows::ffi::OsStrExt};
    s.encode_wide().chain(once(0)).collect()
}

pub fn named_mapping_write_read(
    name: &str,
    size: usize,
    offset: usize,
    data: &[u8],
) -> Result<Vec<u8>> {
    if size == 0 {
        return Err(Error::from_hresult(E_INVALIDARG));
    }
    if offset + data.len() > size {
        return Err(Error::from_hresult(E_INVALIDARG));
    }

    let name_wide = wide_null(OsStr::new(name));

    // Create the file mapping
    let mapping_handle = unsafe {
        CreateFileMappingW(
            INVALID_HANDLE_VALUE,
            None,
            PAGE_READWRITE,
            (size >> 32) as u32,
            size as u32,
            windows::core::PCWSTR(name_wide.as_ptr()),
        )?
    };

    // Ensure we close the handle when done
    struct HandleGuard(HANDLE);
    impl Drop for HandleGuard {
        fn drop(&mut self) {
            if self.0 != INVALID_HANDLE_VALUE {
                unsafe {
                    let _ = CloseHandle(self.0);
                }
            }
        }
    }
    let mapping_guard = HandleGuard(mapping_handle);

    // Map view for writing
    let write_view = unsafe { MapViewOfFile(mapping_guard.0, FILE_MAP_WRITE, 0, 0, size) };
    if write_view.Value.is_null() {
        return Err(Error::from_thread());
    }

    // Write data at offset
    unsafe {
        let dest = (write_view.Value as *mut u8).add(offset);
        std::ptr::copy_nonoverlapping(data.as_ptr(), dest, data.len());
    }

    // Unmap write view
    unsafe {
        UnmapViewOfFile(write_view)?;
    }

    // Open the same mapping for reading
    let read_mapping_handle = unsafe {
        OpenFileMappingW(
            FILE_MAP_READ.0,
            false,
            windows::core::PCWSTR(name_wide.as_ptr()),
        )?
    };

    let read_mapping_guard = HandleGuard(read_mapping_handle);

    // Map view for reading
    let read_view = unsafe { MapViewOfFile(read_mapping_guard.0, FILE_MAP_READ, 0, 0, size) };
    if read_view.Value.is_null() {
        return Err(Error::from_thread());
    }

    // Read data from offset
    let result = unsafe {
        let src = (read_view.Value as *const u8).add(offset);
        let mut buffer = vec![0u8; data.len()];
        std::ptr::copy_nonoverlapping(src, buffer.as_mut_ptr(), data.len());
        buffer
    };

    // Unmap read view
    unsafe {
        UnmapViewOfFile(read_view)?;
    }

    Ok(result)
}