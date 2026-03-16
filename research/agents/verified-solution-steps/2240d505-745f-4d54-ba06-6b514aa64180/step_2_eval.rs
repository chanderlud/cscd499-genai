use std::path::Path;
use windows::core::{Result, Error, PCWSTR};
use windows::Win32::Foundation::{HANDLE, INVALID_HANDLE_VALUE, CloseHandle, GENERIC_READ};
use windows::Win32::Storage::FileSystem::{
    CreateFileW, GetFileInformationByHandleEx, FILE_ID_INFO, FileIdInfo,
    FILE_FLAG_BACKUP_SEMANTICS, FILE_SHARE_READ, FILE_SHARE_WRITE, FILE_SHARE_DELETE,
    OPEN_EXISTING, FILE_ATTRIBUTE_NORMAL,
};

fn wide_null(s: &std::ffi::OsStr) -> Vec<u16> {
    use std::{iter::once, os::windows::ffi::OsStrExt};
    s.encode_wide().chain(once(0)).collect()
}

pub fn file_id(path: &Path) -> Result<[u8; 16]> {
    let wide_path = wide_null(path.as_os_str());
    
    // SAFETY: CreateFileW is called with valid parameters
    let handle = unsafe {
        CreateFileW(
            PCWSTR(wide_path.as_ptr()),
            GENERIC_READ.0,
            FILE_SHARE_READ | FILE_SHARE_WRITE | FILE_SHARE_DELETE,
            None,
            OPEN_EXISTING,
            FILE_ATTRIBUTE_NORMAL | FILE_FLAG_BACKUP_SEMANTICS,
            None,
        )
    }?;
    
    // Ensure handle is closed even if we return early
    struct HandleGuard(HANDLE);
    impl Drop for HandleGuard {
        fn drop(&mut self) {
            // SAFETY: Handle is valid and we're in Drop
            unsafe { CloseHandle(self.0); }
        }
    }
    let _guard = HandleGuard(handle);
    
    let mut file_id_info = FILE_ID_INFO::default();
    
    // SAFETY: GetFileInformationByHandleEx is called with valid parameters
    unsafe {
        GetFileInformationByHandleEx(
            handle,
            FileIdInfo,
            &mut file_id_info as *mut _ as *mut std::ffi::c_void,
            std::mem::size_of::<FILE_ID_INFO>() as u32,
        )
    }?;
    
    // Convert the 128-bit FileId to a 16-byte array
    Ok(file_id_info.FileId.Identifier)
}