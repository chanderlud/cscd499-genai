use windows::core::{Error, Result};
use windows::Win32::Foundation::WIN32_ERROR;
use windows::Win32::System::Memory::{AddSecureMemoryCacheCallback, PSECURE_MEMORY_CACHE_CALLBACK};

fn call_add_secure_memory_cache_callback() -> WIN32_ERROR {
    let callback: PSECURE_MEMORY_CACHE_CALLBACK = None;
    match unsafe { AddSecureMemoryCacheCallback(callback) } {
        Ok(()) => WIN32_ERROR(0),
        Err(e) => WIN32_ERROR(e.code().0 as u32),
    }
}
