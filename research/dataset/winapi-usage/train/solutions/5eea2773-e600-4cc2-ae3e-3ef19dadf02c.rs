use windows::core::{Error, Result};
use windows::Win32::System::Memory::AddSecureMemoryCacheCallback;

unsafe extern "system" fn secure_memory_cache_callback(
    _addr: *const core::ffi::c_void,
    _range: usize,
) -> bool {
    true
}

fn call_add_secure_memory_cache_callback() -> Result<Result<()>> {
    let callback = Some(
        secure_memory_cache_callback
            as unsafe extern "system" fn(*const core::ffi::c_void, usize) -> bool,
    );
    Ok(unsafe { AddSecureMemoryCacheCallback(callback) })
}
