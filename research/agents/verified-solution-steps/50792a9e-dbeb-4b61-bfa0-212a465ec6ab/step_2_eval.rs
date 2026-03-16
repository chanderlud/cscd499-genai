use std::path::Path;
use windows::core::{Result, PCWSTR};
use windows::Win32::Foundation::{CloseHandle, HANDLE, INVALID_HANDLE_VALUE};
use windows::Win32::Storage::FileSystem::{
    CreateFileW, FILE_ATTRIBUTE_NORMAL, FILE_GENERIC_WRITE, FILE_SHARE_READ,
    CREATE_ALWAYS, FILE_STANDARD_INFO, FileStandardInfo, GetFileInformationByHandleEx,
    SetEndOfFile, SetFilePointerEx, WriteFile, FILE_END,
};
use windows::Win32::System::IO::DeviceIoControl;
use windows::Win32::System::Ioctl::{
    FILE_ZERO_DATA_INFORMATION, FSCTL_SET_SPARSE, FSCTL_SET_ZERO_DATA,
};

fn wide_null(s: &std::ffi::OsStr) -> Vec<u16> {
    use std::{iter::once, os::windows::ffi::OsStrExt};
    s.encode_wide().chain(once(0)).collect()
}

pub fn sparse_file_stats(
    path: &Path,
    logical_size: u64,
    hole_start: u64,
    hole_len: u64,
    tail: &[u8],
) -> Result<(u64, u64)> {
    // Create file with write access
    let wide_path = wide_null(path.as_os_str());
    let handle = unsafe {
        CreateFileW(
            PCWSTR(wide_path.as_ptr()),
            FILE_GENERIC_WRITE.0,
            FILE_SHARE_READ,
            None,
            CREATE_ALWAYS,
            FILE_ATTRIBUTE_NORMAL,
            None,
        )?
    };
    let handle = HandleGuard(handle);

    // Set file as sparse
    let mut bytes_returned = 0u32;
    unsafe {
        DeviceIoControl(
            handle.0,
            FSCTL_SET_SPARSE,
            None,
            0,
            None,
            0,
            Some(&mut bytes_returned),
            None,
        )?;
    }

    // Set file size to logical_size
    unsafe {
        SetFilePointerEx(handle.0, logical_size as i64, None, FILE_END)?;
        SetEndOfFile(handle.0)?;
    }

    // Punch hole if hole_len > 0
    if hole_len > 0 {
        let zero_data = FILE_ZERO_DATA_INFORMATION {
            FileOffset: hole_start as i64,
            BeyondFinalZero: (hole_start + hole_len) as i64,
        };
        let mut bytes_returned = 0u32;
        unsafe {
            DeviceIoControl(
                handle.0,
                FSCTL_SET_ZERO_DATA,
                Some(&zero_data as *const _ as *const _),
                std::mem::size_of::<FILE_ZERO_DATA_INFORMATION>() as u32,
                None,
                0,
                Some(&mut bytes_returned),
                None,
            )?;
        }
    }

    // Write tail data at end of file
    if !tail.is_empty() {
        let write_offset = logical_size - tail.len() as u64;
        unsafe {
            SetFilePointerEx(handle.0, write_offset as i64, None, FILE_END)?;
        }
        let mut bytes_written = 0u32;
        unsafe {
            WriteFile(
                handle.0,
                Some(tail),
                Some(&mut bytes_written),
                None,
            )?;
        }
    }

    // Get file allocation size
    let mut file_info = FILE_STANDARD_INFO::default();
    unsafe {
        GetFileInformationByHandleEx(
            handle.0,
            FileStandardInfo,
            &mut file_info as *mut _ as *mut _,
            std::mem::size_of::<FILE_STANDARD_INFO>() as u32,
        )?;
    }

    Ok((logical_size, file_info.AllocationSize as u64))
}

struct HandleGuard(HANDLE);
impl Drop for HandleGuard {
    fn drop(&mut self) {
        unsafe {
            let _ = CloseHandle(self.0);
        }
    }
}