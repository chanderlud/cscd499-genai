use std::ffi::OsStr;
use std::iter::once;
use std::os::windows::ffi::OsStrExt;
use windows::core::{Result, PCWSTR};
use windows::Win32::Foundation::{CloseHandle, HANDLE};
use windows::Win32::Storage::FileSystem::{
    CreateFileW, GetFileSizeEx, ReadFile, SetFilePointerEx, WriteFile, CREATE_ALWAYS,
    FILE_ATTRIBUTE_NORMAL, FILE_BEGIN, FILE_GENERIC_READ, FILE_GENERIC_WRITE, FILE_SHARE_NONE,
    FILE_SHARE_READ, OPEN_EXISTING,
};

fn wide_null(s: &OsStr) -> Vec<u16> {
    s.encode_wide().chain(once(0)).collect()
}

fn open_file_read(path: &OsStr) -> Result<HANDLE> {
    let wide_path = wide_null(path);
    unsafe {
        CreateFileW(
            PCWSTR(wide_path.as_ptr()),
            FILE_GENERIC_READ.0,
            FILE_SHARE_READ,
            None,
            OPEN_EXISTING,
            FILE_ATTRIBUTE_NORMAL,
            None,
        )
    }
}

fn open_file_write(path: &OsStr) -> Result<HANDLE> {
    let wide_path = wide_null(path);
    unsafe {
        CreateFileW(
            PCWSTR(wide_path.as_ptr()),
            FILE_GENERIC_WRITE.0,
            FILE_SHARE_NONE,
            None,
            CREATE_ALWAYS,
            FILE_ATTRIBUTE_NORMAL,
            None,
        )
    }
}

fn get_file_size(handle: HANDLE) -> Result<i64> {
    let mut size = 0i64;
    unsafe { GetFileSizeEx(handle, &mut size) }?;
    Ok(size)
}

fn set_file_position(handle: HANDLE, position: i64) -> Result<()> {
    unsafe { SetFilePointerEx(handle, position, None, FILE_BEGIN) }?;
    Ok(())
}

fn read_chunk(handle: HANDLE, buffer: &mut [u8]) -> Result<u32> {
    let mut bytes_read = 0u32;
    unsafe { ReadFile(handle, Some(buffer), Some(&mut bytes_read), None) }?;
    Ok(bytes_read)
}

fn write_chunk(handle: HANDLE, buffer: &[u8]) -> Result<()> {
    let mut bytes_written = 0u32;
    unsafe { WriteFile(handle, Some(buffer), Some(&mut bytes_written), None) }?;
    Ok(())
}

pub fn reverse_file_chunks(src_path: &str, dst_path: &str) -> Result<()> {
    const CHUNK_SIZE: usize = 4096;

    let src_handle = open_file_read(OsStr::new(src_path))?;
    let dst_handle = open_file_write(OsStr::new(dst_path))?;

    // Ensure handles are closed on any error
    let result = (|| {
        let file_size = get_file_size(src_handle)?;
        let total_chunks = (file_size as usize + CHUNK_SIZE - 1) / CHUNK_SIZE;

        let mut buffer = vec![0u8; CHUNK_SIZE];

        // Process chunks in reverse order
        for chunk_index in (0..total_chunks).rev() {
            let offset = (chunk_index * CHUNK_SIZE) as i64;
            set_file_position(src_handle, offset)?;

            // Calculate bytes to read for this chunk
            let bytes_to_read = if chunk_index == total_chunks - 1 {
                // Last chunk might be smaller
                let remaining = file_size as usize - (chunk_index * CHUNK_SIZE);
                remaining.min(CHUNK_SIZE)
            } else {
                CHUNK_SIZE
            };

            let bytes_read = read_chunk(src_handle, &mut buffer[..bytes_to_read])?;
            write_chunk(dst_handle, &buffer[..bytes_read as usize])?;
        }

        Ok(())
    })();

    // SAFETY: Closing handles that were successfully opened
    unsafe {
        let _ = CloseHandle(dst_handle);
        let _ = CloseHandle(src_handle);
    }

    result
}
