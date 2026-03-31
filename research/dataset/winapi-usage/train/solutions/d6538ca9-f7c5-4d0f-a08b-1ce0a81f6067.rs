#![deny(warnings)]

#[allow(unused_imports)]
use windows::core::{Error, Result};
use windows::Win32::Foundation::{FreeLibrary, HMODULE};

#[allow(dead_code)]
fn call_free_library() -> Result<Result<()>> {
    let hmodule = HMODULE::default();
    // SAFETY: FreeLibrary is safe to call with a default/null HMODULE.
    // It will return an Err if the handle is invalid, which is propagated correctly.
    unsafe { Ok(FreeLibrary(hmodule)) }
}
