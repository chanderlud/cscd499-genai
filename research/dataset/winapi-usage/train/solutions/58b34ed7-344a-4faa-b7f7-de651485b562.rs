use std::ffi::c_void;
use windows::core::{Error, Result};
use windows::Win32::System::Search::SQLAllocConnect;

fn call_sql_alloc_connect() -> Result<i16> {
    let mut connection_handle: *mut c_void = std::ptr::null_mut();
    // SAFETY: SQLAllocConnect is an unsafe FFI function. We pass valid pointers.
    let result = unsafe { SQLAllocConnect(std::ptr::null_mut(), &mut connection_handle) };
    Ok(result)
}
