use std::ffi::OsStr;
use std::iter::once;
use std::os::windows::ffi::OsStrExt;
use windows::core::{Error, Result, HRESULT, PCWSTR};
use windows::Win32::Foundation::{
    CloseHandle, ERROR_INVALID_PARAMETER, ERROR_WRITE_FAULT, GENERIC_READ, GENERIC_WRITE, HANDLE,
};
use windows::Win32::Storage::FileSystem::{
    CreateFileW, ReadFile, WriteFile, CREATE_ALWAYS, FILE_ATTRIBUTE_NORMAL, FILE_SHARE_NONE,
    FILE_SHARE_READ, OPEN_EXISTING,
};

fn wide_null(s: &OsStr) -> Vec<u16> {
    s.encode_wide().chain(once(0)).collect()
}

struct FileHandle(HANDLE);

impl FileHandle {
    fn open_read(path: &OsStr) -> Result<Self> {
        let wide_path = wide_null(path);
        // SAFETY: FFI call with valid parameters
        let handle = unsafe {
            CreateFileW(
                PCWSTR(wide_path.as_ptr()),
                GENERIC_READ.0,
                FILE_SHARE_READ,
                None,
                OPEN_EXISTING,
                FILE_ATTRIBUTE_NORMAL,
                None,
            )?
        };
        Ok(Self(handle))
    }

    fn create_write(path: &OsStr) -> Result<Self> {
        let wide_path = wide_null(path);
        // SAFETY: FFI call with valid parameters
        let handle = unsafe {
            CreateFileW(
                PCWSTR(wide_path.as_ptr()),
                GENERIC_WRITE.0,
                FILE_SHARE_NONE,
                None,
                CREATE_ALWAYS,
                FILE_ATTRIBUTE_NORMAL,
                None,
            )?
        };
        Ok(Self(handle))
    }

    fn read_chunk(&self, buffer: &mut [u8]) -> Result<u32> {
        let mut bytes_read = 0u32;
        // SAFETY: FFI call with valid buffer pointer and size
        unsafe {
            ReadFile(self.0, Some(buffer), Some(&mut bytes_read), None)?;
        }
        Ok(bytes_read)
    }

    fn write_chunk(&self, buffer: &[u8]) -> Result<()> {
        let mut bytes_written = 0u32;
        // SAFETY: FFI call with valid buffer pointer and size
        unsafe {
            WriteFile(self.0, Some(buffer), Some(&mut bytes_written), None)?;
        }

        if bytes_written as usize != buffer.len() {
            // Partial write
            Err(Error::from(ERROR_WRITE_FAULT))
        } else {
            Ok(())
        }
    }
}

impl Drop for FileHandle {
    fn drop(&mut self) {
        // SAFETY: Closing a valid handle
        unsafe { CloseHandle(self.0) };
    }
}

pub fn merge_files_alternating(
    path1: &str,
    path2: &str,
    output_path: &str,
    chunk_size: u32,
) -> Result<()> {
    if chunk_size == 0 {
        return Err(Error::from(ERROR_INVALID_PARAMETER));
    }

    let file1 = FileHandle::open_read(OsStr::new(path1))?;
    let file2 = FileHandle::open_read(OsStr::new(path2))?;
    let output = FileHandle::create_write(OsStr::new(output_path))?;

    let mut buffer1 = vec![0u8; chunk_size as usize];
    let mut buffer2 = vec![0u8; chunk_size as usize];
    let mut file1_done = false;
    let mut file2_done = false;

    while !file1_done || !file2_done {
        if !file1_done {
            match file1.read_chunk(&mut buffer1)? {
                0 => file1_done = true,
                bytes_read => {
                    output.write_chunk(&buffer1[..bytes_read as usize])?;
                }
            }
        }

        if !file2_done {
            match file2.read_chunk(&mut buffer2)? {
                0 => file2_done = true,
                bytes_read => {
                    output.write_chunk(&buffer2[..bytes_read as usize])?;
                }
            }
        }
    }

    Ok(())
}
