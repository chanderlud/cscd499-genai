use windows::core::HRESULT;
use windows::Win32::Foundation::S_OK;
use windows::Win32::Networking::WinInet::AppCacheCloseHandle;

fn call_app_cache_close_handle() -> HRESULT {
    // SAFETY: AppCacheCloseHandle is a Win32 API that expects a handle pointer.
    // We pass a null pointer as a concrete value for this exercise.
    unsafe {
        AppCacheCloseHandle(std::ptr::null());
    }
    S_OK
}
