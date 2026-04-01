use windows::core::HRESULT;
use windows::Win32::Foundation::S_OK;
use windows::Win32::Networking::WinInet::AppCacheDeleteIEGroup;

fn call_app_cache_delete_ie_group() -> HRESULT {
    // Create a concrete manifest URL string (null-terminated UTF-16)
    let manifest_url = "http://example.com/app.manifest\0";
    let manifest_url_wide: Vec<u16> = manifest_url.encode_utf16().collect();

    // Call the API - returns u32 (WIN32_ERROR code)
    let result = unsafe {
        AppCacheDeleteIEGroup(windows::core::PCWSTR::from_raw(manifest_url_wide.as_ptr()))
    };

    // Check if successful (0 = success)
    if result == 0 {
        S_OK
    } else {
        // Convert WIN32_ERROR to HRESULT
        HRESULT::from_win32(result)
    }
}
