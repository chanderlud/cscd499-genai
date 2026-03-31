use windows::core::{Error, Result, HRESULT};
use windows::Win32::Storage::FileSystem::CreateDirectoryW;

fn call_create_directory_w() -> HRESULT {
    unsafe {
        CreateDirectoryW(windows::core::w!("test_dir"), None)
            .map(|_| HRESULT(0))
            .unwrap_or_else(|e| e.code())
    }
}
