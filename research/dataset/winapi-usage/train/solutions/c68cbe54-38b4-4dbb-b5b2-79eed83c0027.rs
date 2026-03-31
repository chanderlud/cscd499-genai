use windows::Win32::Foundation::{FreeLibrary, HMODULE, WIN32_ERROR};

fn call_free_library() -> WIN32_ERROR {
    let result = unsafe { FreeLibrary(HMODULE::default()) };
    match result {
        Ok(()) => WIN32_ERROR(0),
        Err(e) => WIN32_ERROR::from_error(&e).unwrap_or_default(),
    }
}
