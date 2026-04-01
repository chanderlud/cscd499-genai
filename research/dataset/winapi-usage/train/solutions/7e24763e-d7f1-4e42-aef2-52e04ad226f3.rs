use windows::core::{Error, Result};
use windows::Win32::Security::Credentials::CredFree;

fn call_cred_free() -> windows::core::Result<()> {
    // Create a concrete buffer to pass to CredFree
    // CredFree expects a pointer to memory allocated by Cred functions
    let buffer = Box::new([0u8; 1024]);
    let ptr = Box::into_raw(buffer) as *const core::ffi::c_void;

    // Call CredFree to free the buffer
    // This is safe because we own the pointer and CredFree will free it
    unsafe {
        CredFree(ptr);
    }

    Ok(())
}
