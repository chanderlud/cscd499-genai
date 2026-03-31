use windows::Win32::Foundation::WIN32_ERROR;
use windows::Win32::System::LibraryLoader::GetModuleHandleW;

fn call_get_module_handle_w() -> WIN32_ERROR {
    // SAFETY: Passing None is valid and safely retrieves the handle to the calling module.
    match unsafe { GetModuleHandleW(None) } {
        Ok(_) => WIN32_ERROR(0),
        Err(e) => WIN32_ERROR::from_error(&e).unwrap_or_default(),
    }
}
