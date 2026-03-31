use windows::core::{Error, Result, PCWSTR};
use windows::Win32::Foundation::WIN32_ERROR;
use windows::Win32::Networking::WinInet::AppCacheCreateAndCommitFile;

fn call_app_cache_create_and_commit_file() -> WIN32_ERROR {
    unsafe {
        let code =
            AppCacheCreateAndCommitFile(std::ptr::null(), PCWSTR::null(), PCWSTR::null(), &[]);
        WIN32_ERROR(code)
    }
}
