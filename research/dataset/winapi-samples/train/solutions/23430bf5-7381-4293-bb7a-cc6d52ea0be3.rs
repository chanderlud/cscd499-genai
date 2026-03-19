use std::io;
use std::os::windows::ffi::OsStrExt;
use std::path::Path;
use windows::core::{Error, PCWSTR};
use windows::Win32::Foundation::CloseHandle;
use windows::Win32::Foundation::GENERIC_READ;
use windows::Win32::Storage::FileSystem::{
    CreateFileW, GetFileSizeEx, FILE_FLAGS_AND_ATTRIBUTES, FILE_SHARE_READ, OPEN_EXISTING,
};
use windows::Win32::System::Memory::{
    CreateFileMappingW, MapViewOfFile, UnmapViewOfFile, FILE_MAP_READ, PAGE_READONLY,
};

fn to_wide_fixed(path: &Path, buf: &mut [u16; 260]) -> io::Result<PCWSTR> {
    let os_str = path.as_os_str();
    let iter = os_str.encode_wide();
    let mut i = 0;
    for w in iter {
        if i >= buf.len() - 1 {
            return Err(io::Error::new(io::ErrorKind::InvalidInput, "Path too long"));
        }
        buf[i] = w;
        i += 1;
    }
    buf[i] = 0;
    Ok(PCWSTR(buf.as_ptr()))
}

fn count_nonoverlapping(data: &[u8], needle: &[u8]) -> usize {
    if needle.is_empty() {
        return 0;
    }
    let mut count = 0;
    let mut i = 0;
    while i <= data.len() - needle.len() {
        if &data[i..i + needle.len()] == needle {
            count += 1;
            i += needle.len();
        } else {
            i += 1;
        }
    }
    count
}

pub fn mmap_count_nonoverlapping(path: &Path, needle: &[u8]) -> io::Result<usize> {
    let mut wide_buf = [0u16; 260];
    let wide_path = to_wide_fixed(path, &mut wide_buf)?;

    // Open file for reading
    let file_handle = unsafe {
        CreateFileW(
            wide_path,
            GENERIC_READ.0, // Convert GENERIC_ACCESS_RIGHTS to u32
            FILE_SHARE_READ,
            Some(std::ptr::null()),
            OPEN_EXISTING,
            FILE_FLAGS_AND_ATTRIBUTES(0),
            None,
        )
    }?;

    // Create file mapping
    let mapping_handle = unsafe {
        CreateFileMappingW(
            file_handle,
            Some(std::ptr::null()),
            PAGE_READONLY,
            0,
            0,
            PCWSTR::null(),
        )
    }?;

    // Map view of file - returns MEMORY_MAPPED_VIEW_ADDRESS, not Result
    let view = unsafe { MapViewOfFile(mapping_handle, FILE_MAP_READ, 0, 0, 0) };

    // Check for null pointer
    if view.Value.is_null() {
        let err = Error::from_thread();
        unsafe {
            let _ = CloseHandle(mapping_handle);
            let _ = CloseHandle(file_handle);
        }
        return Err(err.into());
    }

    // Get file size using 64-bit function
    let mut file_size: i64 = 0;
    unsafe { GetFileSizeEx(file_handle, &mut file_size) }?;

    // Create slice from mapped memory
    let data = unsafe { std::slice::from_raw_parts(view.Value as *const u8, file_size as usize) };

    // Count occurrences
    let count = count_nonoverlapping(data, needle);

    // Cleanup
    unsafe {
        UnmapViewOfFile(view)?;
        CloseHandle(mapping_handle)?;
        CloseHandle(file_handle)?;
    }

    Ok(count)
}
