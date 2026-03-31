use windows::core::{Error, Result, HRESULT};
use windows::Win32::Networking::WinInet::AppCacheCreateAndCommitFile;

fn call_app_cache_create_and_commit_file() -> HRESULT {
    let code = unsafe {
        AppCacheCreateAndCommitFile(
            std::ptr::null(),
            windows::core::w!("source"),
            windows::core::w!("url"),
            &[],
        )
    };
    HRESULT::from_win32(code)
}
