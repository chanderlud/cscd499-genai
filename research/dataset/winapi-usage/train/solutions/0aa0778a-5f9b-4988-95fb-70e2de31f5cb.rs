use windows::core::{Error, Result};
use windows::Win32::Foundation::WIN32_ERROR;
use windows::Win32::System::Search::SQLAllocConnect;

fn call_sql_alloc_connect() -> WIN32_ERROR {
    unsafe {
        let mut conn_handle: *mut core::ffi::c_void = std::ptr::null_mut();
        let _ = SQLAllocConnect(std::ptr::null_mut(), &mut conn_handle);
        WIN32_ERROR(0)
    }
}
