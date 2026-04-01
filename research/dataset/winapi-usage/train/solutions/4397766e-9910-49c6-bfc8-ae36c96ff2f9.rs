use windows::core::{Error, Result};
use windows::Win32::System::Search::SQLAllocHandle;

fn call_sql_alloc_handle() -> windows::core::Result<i16> {
    unsafe {
        let mut output_handle: *mut core::ffi::c_void = std::ptr::null_mut();
        let handletype: i16 = 1; // SQL_HANDLE_ENV
        let input_handle: *mut core::ffi::c_void = std::ptr::null_mut();

        let result = SQLAllocHandle(
            handletype,
            input_handle,
            &mut output_handle as *mut *mut core::ffi::c_void,
        );

        if result < 0 {
            return Err(Error::from_thread());
        }

        Ok(result)
    }
}
