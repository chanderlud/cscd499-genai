#![deny(warnings)]

use windows::core::HRESULT;
use windows::Win32::Foundation::{FreeLibrary, HMODULE};

#[allow(dead_code)]
fn call_free_library() -> HRESULT {
    // SAFETY: Passing a null HMODULE is safe for demonstration; FreeLibrary will fail gracefully.
    unsafe {
        match FreeLibrary(HMODULE(std::ptr::null_mut())) {
            Ok(()) => HRESULT(0),
            Err(e) => e.code(),
        }
    }
}
