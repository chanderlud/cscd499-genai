use std::ffi::OsStr;
use std::iter;
use std::os::windows::ffi::OsStrExt;
use std::path::PathBuf;

use windows::core::{Error, Result, PCWSTR};
use windows::Win32::Foundation::{CloseHandle, GENERIC_WRITE};
use windows::Win32::Storage::FileSystem::{
    CreateFileW, FlushFileBuffers, GetTempPathW, SetFileAttributesW, WriteFile, CREATE_ALWAYS,
    FILE_ATTRIBUTE_NORMAL, FILE_ATTRIBUTE_READONLY, FILE_SHARE_READ,
};

pub struct TempFileWriteResult {
    pub path: PathBuf,
    pub bytes_written: u32,
}

fn to_wide_null(value: &OsStr) -> Vec<u16> {
    value.encode_wide().chain(iter::once(0)).collect()
}

pub fn write_readonly_temp_file(stem: &str, contents: &[u8]) -> Result<TempFileWriteResult> {
    let mut temp_buf = vec![0u16; 261];
    let temp_len = unsafe { GetTempPathW(Some(&mut temp_buf)) } as usize;
    if temp_len == 0 {
        return Err(Error::from_thread());
    }

    let temp_dir = String::from_utf16_lossy(&temp_buf[..temp_len]);
    let mut path = PathBuf::from(temp_dir);
    path.push(format!("{stem}.txt"));

    let wide_path = to_wide_null(path.as_os_str());

    let handle = unsafe {
        CreateFileW(
            PCWSTR(wide_path.as_ptr()),
            GENERIC_WRITE.0,
            FILE_SHARE_READ,
            None,
            CREATE_ALWAYS,
            FILE_ATTRIBUTE_NORMAL,
            None,
        )?
    };

    let write_result = (|| -> Result<u32> {
        let mut total_written = 0u32;

        while (total_written as usize) < contents.len() {
            let mut just_written = 0u32;

            unsafe {
                WriteFile(
                    handle,
                    Some(&contents[total_written as usize..]),
                    Some(&mut just_written as *mut u32),
                    None,
                )?;
            }

            if just_written == 0 {
                return Err(Error::from_thread());
            }

            total_written += just_written;
        }

        unsafe {
            FlushFileBuffers(handle)?;
        }

        Ok(total_written)
    })();

    let close_result = unsafe { CloseHandle(handle) };

    let bytes_written = write_result?;
    close_result?;

    unsafe {
        SetFileAttributesW(PCWSTR(wide_path.as_ptr()), FILE_ATTRIBUTE_READONLY)?;
    }

    Ok(TempFileWriteResult {
        path,
        bytes_written,
    })
}
