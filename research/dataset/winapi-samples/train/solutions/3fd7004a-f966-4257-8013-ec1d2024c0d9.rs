use std::ffi::OsStr;
use std::iter::once;
use std::os::windows::ffi::OsStrExt;
use windows::core::{Error, Result, HRESULT, PCWSTR};
use windows::Win32::Foundation::{
    CloseHandle, ERROR_HANDLE_EOF, ERROR_INVALID_PARAMETER, GENERIC_READ, HANDLE,
};
use windows::Win32::Storage::FileSystem::{
    CreateFileW, GetFileSizeEx, ReadFile, WriteFile, CREATE_ALWAYS, FILE_ATTRIBUTE_NORMAL,
    FILE_GENERIC_WRITE, FILE_SHARE_MODE, FILE_SHARE_READ, OPEN_EXISTING,
};

fn wide_null(s: &OsStr) -> Vec<u16> {
    s.encode_wide().chain(once(0)).collect()
}

fn open_file_read(path: &OsStr) -> Result<HANDLE> {
    let wide_path = wide_null(path);
    unsafe {
        CreateFileW(
            PCWSTR(wide_path.as_ptr()),
            GENERIC_READ.0,
            FILE_SHARE_READ,
            None,
            OPEN_EXISTING,
            FILE_ATTRIBUTE_NORMAL,
            None,
        )
    }
}

fn create_file_write(path: &OsStr) -> Result<HANDLE> {
    let wide_path = wide_null(path);
    unsafe {
        CreateFileW(
            PCWSTR(wide_path.as_ptr()),
            FILE_GENERIC_WRITE.0,
            FILE_SHARE_MODE(0),
            None,
            CREATE_ALWAYS,
            FILE_ATTRIBUTE_NORMAL,
            None,
        )
    }
}

fn get_file_size(handle: HANDLE) -> Result<u64> {
    let mut size = 0i64;
    unsafe {
        GetFileSizeEx(handle, &mut size)?;
    }
    Ok(size as u64)
}

fn read_file(handle: HANDLE, buffer: &mut [u8]) -> Result<u32> {
    let mut bytes_read = 0u32;
    unsafe {
        ReadFile(handle, Some(buffer), Some(&mut bytes_read), None)?;
    }
    Ok(bytes_read)
}

fn write_file(handle: HANDLE, buffer: &[u8]) -> Result<()> {
    let mut bytes_written = 0u32;
    unsafe {
        WriteFile(handle, Some(buffer), Some(&mut bytes_written), None)?;
    }

    if bytes_written == buffer.len() as u32 {
        Ok(())
    } else {
        Err(Error::from_hresult(HRESULT::from_win32(ERROR_HANDLE_EOF.0)))
    }
}

fn close_handle(handle: HANDLE) {
    let _ = unsafe { CloseHandle(handle) };
}

pub fn split_file_into_parts(source_path: &str, num_parts: u32) -> Result<u32> {
    if num_parts == 0 {
        return Err(Error::from_hresult(HRESULT::from_win32(
            ERROR_INVALID_PARAMETER.0,
        )));
    }

    let source_os = OsStr::new(source_path);
    let source_handle = open_file_read(source_os)?;

    // Ensure source handle is closed on error
    let result = (|| -> Result<u32> {
        let file_size = get_file_size(source_handle)?;

        if file_size == 0 {
            // Create one empty part for empty file
            let part_path = format!("{}_part0", source_path);
            let part_handle = create_file_write(OsStr::new(&part_path))?;
            close_handle(part_handle);
            return Ok(1);
        }

        // Calculate aligned part size (must be multiple of 512)
        let raw_part_size = file_size / num_parts as u64;
        let aligned_part_size = (raw_part_size / 512) * 512;

        if aligned_part_size == 0 {
            return Err(Error::from_hresult(HRESULT::from_win32(
                ERROR_INVALID_PARAMETER.0,
            )));
        }

        let mut buffer = vec![0u8; 4096]; // 4KB buffer for reading/writing
        let mut current_offset = 0u64;

        for part_index in 0..num_parts {
            let part_path = format!("{}_part{}", source_path, part_index);
            let part_handle = create_file_write(OsStr::new(&part_path))?;

            // Ensure part handle is closed on error
            let part_result = (|| -> Result<()> {
                // Calculate bytes to write for this part
                let bytes_to_write = if part_index < num_parts - 1 {
                    aligned_part_size
                } else {
                    // Last part gets remaining bytes
                    file_size - current_offset
                };

                let mut bytes_written = 0u64;

                while bytes_written < bytes_to_write {
                    let bytes_to_read =
                        std::cmp::min(buffer.len() as u64, bytes_to_write - bytes_written) as usize;

                    let bytes_read = read_file(source_handle, &mut buffer[..bytes_to_read])?;
                    if bytes_read == 0 {
                        break; // EOF reached
                    }

                    write_file(part_handle, &buffer[..bytes_read as usize])?;
                    bytes_written += bytes_read as u64;
                    current_offset += bytes_read as u64;
                }

                Ok(())
            })();

            close_handle(part_handle);
            part_result?;
        }

        Ok(num_parts)
    })();

    close_handle(source_handle);
    result
}
