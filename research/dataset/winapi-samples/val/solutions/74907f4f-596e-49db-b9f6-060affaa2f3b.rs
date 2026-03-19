use windows::{
    core::{PCSTR, PCWSTR},
    Win32::{
        Foundation::FreeLibrary,
        Globalization::{WideCharToMultiByte, CP_ACP},
        System::LibraryLoader::{GetProcAddress, LoadLibraryExW, LOAD_LIBRARY_SEARCH_DEFAULT_DIRS},
    },
};

/// Converts a wide string (PCWSTR) to a null-terminated ANSI string (Vec<u8>).
/// Returns None if conversion fails.
fn wide_to_ansi(wide: PCWSTR) -> Option<Vec<u8>> {
    // SAFETY: The caller must ensure `wide` points to a valid null-terminated UTF-16 string.
    unsafe {
        // Convert PCWSTR to &[u16] slice
        let wide_slice = wide.as_wide();

        // First call: get required buffer size
        let len = WideCharToMultiByte(CP_ACP, 0, wide_slice, None, None, None);
        if len == 0 {
            return None;
        }

        // Allocate buffer and convert
        let mut ansi = vec![0u8; len as usize];
        let result = WideCharToMultiByte(CP_ACP, 0, wide_slice, Some(&mut ansi), None, None);
        if result == 0 {
            return None;
        }

        Some(ansi)
    }
}

/// Dynamically loads a DLL and retrieves a function pointer by name.
/// Returns None if any step fails.
///
/// # Safety
/// The caller must ensure the function pointer type `T` matches the actual function signature.
pub unsafe fn delay_load_wide<T>(library: PCWSTR, function: PCWSTR) -> Option<T> {
    // Load the library
    // SAFETY: The caller must ensure `library` is a valid null-terminated wide string.
    let hmodule = unsafe {
        match LoadLibraryExW(library, None, LOAD_LIBRARY_SEARCH_DEFAULT_DIRS) {
            Ok(h) => h,
            Err(_) => return None,
        }
    };

    // Convert function name to ANSI
    let ansi_name = match wide_to_ansi(function) {
        Some(name) => name,
        None => {
            // Conversion failed, free library and return None
            // SAFETY: hmodule is valid from LoadLibraryExW
            unsafe { FreeLibrary(hmodule) };
            return None;
        }
    };

    // Get function address
    // SAFETY: hmodule is valid, ansi_name is a valid null-terminated ANSI string.
    let proc = unsafe { GetProcAddress(hmodule, PCSTR(ansi_name.as_ptr())) };

    match proc {
        Some(addr) => {
            // Function found, transmute to requested type
            // SAFETY: The caller must ensure T matches the actual function signature.
            Some(unsafe { std::mem::transmute_copy(&addr) })
        }
        None => {
            // Function not found, free library and return None
            // SAFETY: hmodule is valid from LoadLibraryExW
            unsafe { FreeLibrary(hmodule) };
            None
        }
    }
}
