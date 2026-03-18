use windows::Win32::Foundation::{ERROR_INVALID_PARAMETER, HANDLE, INVALID_HANDLE_VALUE};
use windows::Win32::System::Memory::{CreateFileMappingW, PAGE_READWRITE};
use windows::core::{Error, HRESULT, PCWSTR, Result};

/// A wrapper around a Windows HANDLE that automatically closes it on drop.
/// This struct is NOT Send or Sync because HANDLEs have thread-affinity constraints.
pub struct OwnedHandle(HANDLE);

impl OwnedHandle {
    /// Creates a new OwnedHandle from a raw HANDLE.
    ///
    /// # Safety
    /// The caller must ensure the handle is valid and not already owned by another wrapper.
    pub unsafe fn new(handle: HANDLE) -> Self {
        Self(handle)
    }

    /// Returns the underlying HANDLE.
    pub fn handle(&self) -> HANDLE {
        self.0
    }
}

impl Drop for OwnedHandle {
    fn drop(&mut self) {
        // SAFETY: We own this handle and it was valid when created.
        // CloseHandle is safe to call with a valid handle.
        unsafe {
            let _ = windows::Win32::Foundation::CloseHandle(self.0);
        }
    }
}

/// Converts a Rust string to a null-terminated UTF-16 vector.
fn wide_null(s: &str) -> Vec<u16> {
    use std::iter::once;
    s.encode_utf16().chain(once(0)).collect()
}

/// Creates a named file mapping object for shared memory IPC.
///
/// # Arguments
/// * `name` - The name of the mapping object. Should use "Local\" prefix for session-local objects.
/// * `size` - The size of the mapping in bytes. Must be greater than zero.
///
/// # Returns
/// An `OwnedHandle` that owns the file mapping object. The mapping is closed when the handle is dropped.
///
/// # Errors
/// Returns an error if:
/// - The name contains embedded NUL characters
/// - The size is zero
/// - The system call fails (e.g., due to permissions or name collisions)
pub fn create_named_mapping(name: &str, size: usize) -> Result<OwnedHandle> {
    // Validate inputs
    if name.contains('\0') {
        return Err(Error::from_hresult(HRESULT::from_win32(
            ERROR_INVALID_PARAMETER.0,
        )));
    }

    if size == 0 {
        return Err(Error::from_hresult(HRESULT::from_win32(
            ERROR_INVALID_PARAMETER.0,
        )));
    }

    // Convert name to wide string
    let wide_name = wide_null(name);

    // SAFETY: We're calling a Windows API function with validated parameters.
    // The wide_name is properly null-terminated and we've checked for embedded NULs.
    unsafe {
        let handle = CreateFileMappingW(
            INVALID_HANDLE_VALUE,       // Not backed by a file
            None,                       // Default security
            PAGE_READWRITE,             // Read/write access
            0,                          // High-order size bits (0 for <4GB)
            size as u32,                // Low-order size bits
            PCWSTR(wide_name.as_ptr()), // Name of mapping object
        )?;

        // CreateFileMappingW returns NULL on failure, but the ? operator above
        // will have already converted any error. If we get here, handle is valid.
        Ok(OwnedHandle::new(handle))
    }
}
