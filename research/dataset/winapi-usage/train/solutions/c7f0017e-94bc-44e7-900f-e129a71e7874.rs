use windows::core::{Result, HRESULT};
use windows::Win32::System::Search::SQLAllocHandle;

fn call_sql_alloc_handle() -> HRESULT {
    let mut output_handle: *mut core::ffi::c_void = std::ptr::null_mut();

    // SQL_HANDLE_ENV = 1, SQL_HANDLE_DBC = 2
    let handletype: i16 = 1;
    let inputhandle: *mut core::ffi::c_void = std::ptr::null_mut();

    let ret: i16 = unsafe { SQLAllocHandle(handletype, inputhandle, &mut output_handle) };

    // Convert SQL return code to HRESULT
    // SQL_SUCCESS = 0, SQL_ERROR = -1, etc.
    if ret == 0 {
        HRESULT::from_win32(0)
    } else {
        HRESULT::from_win32(ret as u32)
    }
}
