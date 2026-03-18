use std::ffi::OsStr;
use std::iter::once;
use std::os::windows::ffi::OsStrExt;
use windows::core::{Error, Result, HRESULT, PCWSTR};
use windows::Win32::Foundation::{CloseHandle, GENERIC_READ, GENERIC_WRITE, HANDLE};
use windows::Win32::Storage::FileSystem::{
    CreateFileW, ReadFile, WriteFile, CREATE_ALWAYS, FILE_ATTRIBUTE_NORMAL,
    FILE_FLAGS_AND_ATTRIBUTES, FILE_FLAG_NO_BUFFERING, FILE_FLAG_WRITE_THROUGH, FILE_SHARE_MODE,
    FILE_SHARE_READ, OPEN_EXISTING,
};

fn wide_null(s: &OsStr) -> Vec<u16> {
    s.encode_wide().chain(once(0)).collect()
}

fn open_file_read(path: &OsStr) -> Result<HANDLE> {
    let wide_path = wide_null(path);
    unsafe {
        let handle = CreateFileW(
            PCWSTR(wide_path.as_ptr()),
            GENERIC_READ.0,
            FILE_SHARE_MODE(FILE_SHARE_READ.0),
            None,
            OPEN_EXISTING,
            FILE_FLAGS_AND_ATTRIBUTES(FILE_ATTRIBUTE_NORMAL.0),
            None,
        )?;
        Ok(handle)
    }
}

fn open_file_write(path: &OsStr) -> Result<HANDLE> {
    let wide_path = wide_null(path);
    unsafe {
        let handle = CreateFileW(
            PCWSTR(wide_path.as_ptr()),
            GENERIC_WRITE.0,
            FILE_SHARE_MODE(0),
            None,
            CREATE_ALWAYS,
            FILE_FLAGS_AND_ATTRIBUTES(
                FILE_ATTRIBUTE_NORMAL.0 | FILE_FLAG_NO_BUFFERING.0 | FILE_FLAG_WRITE_THROUGH.0,
            ),
            None,
        )?;
        Ok(handle)
    }
}

fn read_file(handle: HANDLE, buffer: &mut [u8]) -> Result<u32> {
    let mut bytes_read = 0u32;
    unsafe {
        ReadFile(
            handle,
            Some(buffer),
            Some(&mut bytes_read as *mut u32),
            None,
        )?;
    }
    Ok(bytes_read)
}

fn write_file(handle: HANDLE, buffer: &[u8]) -> Result<()> {
    let mut bytes_written = 0u32;
    unsafe {
        WriteFile(
            handle,
            Some(buffer),
            Some(&mut bytes_written as *mut u32),
            None,
        )?;
    }
    Ok(())
}

fn close_file(handle: HANDLE) -> Result<()> {
    unsafe {
        CloseHandle(handle)?;
        Ok(())
    }
}

pub fn split_file_by_size(source_path: &str, part_size: u32) -> Result<u32> {
    if part_size == 0 {
        return Err(Error::from_hresult(HRESULT::from_win32(
            windows::Win32::Foundation::ERROR_INVALID_PARAMETER.0,
        )));
    }

    // Ensure part_size is a multiple of 512 for alignment
    let aligned_part_size = (part_size / 512) * 512;
    if aligned_part_size == 0 {
        return Err(Error::from_hresult(HRESULT::from_win32(
            windows::Win32::Foundation::ERROR_INVALID_PARAMETER.0,
        )));
    }

    let source_os = OsStr::new(source_path);
    let source_handle = open_file_read(source_os)?;

    let mut buffer = vec![0u8; 4096]; // 4KB buffer for reading
    let mut part_index = 0u32;
    let mut current_part_bytes = 0u32;
    let mut current_part_handle = HANDLE::default();

    loop {
        // Create new part file if needed
        if current_part_handle == HANDLE::default() {
            let part_path = format!("{}_part{}", source_path, part_index);
            let part_os = OsStr::new(&part_path);
            current_part_handle = open_file_write(part_os)?;
            current_part_bytes = 0;
        }

        // Read from source
        let bytes_to_read =
            std::cmp::min(buffer.len() as u32, aligned_part_size - current_part_bytes);
        let bytes_read = read_file(source_handle, &mut buffer[..bytes_to_read as usize])?;

        if bytes_read == 0 {
            // End of file
            break;
        }

        // Write to current part
        write_file(current_part_handle, &buffer[..bytes_read as usize])?;
        current_part_bytes += bytes_read;

        // Check if current part is full
        if current_part_bytes >= aligned_part_size {
            close_file(current_part_handle)?;
            current_part_handle = HANDLE::default();
            part_index += 1;
        }
    }

    // Close any open part file
    if current_part_handle != HANDLE::default() {
        close_file(current_part_handle)?;
    }

    // Close source file
    close_file(source_handle)?;

    // Return number of parts created (part_index + 1 if we created at least one part)
    Ok(if part_index == 0 && current_part_bytes == 0 {
        0
    } else {
        part_index + 1
    })
}
