use windows::Win32::Foundation::WIN32_ERROR;
use windows::Win32::System::Search::SQLAllocHandle;

fn call_sql_alloc_handle() -> WIN32_ERROR {
    let mut output_handle: *mut core::ffi::c_void = std::ptr::null_mut();

    // Call SQLAllocHandle with concrete parameter values
    // SQL_HANDLE_ENV = 1, null input handle, pointer to output handle
    let result: i16 = unsafe { SQLAllocHandle(1, std::ptr::null_mut(), &mut output_handle) };

    // Convert SQLRETURN (i16) to WIN32_ERROR
    // SQL_SUCCESS = 0, SQL_ERROR = -1, SQL_SUCCESS_WITH_INFO = 1
    if result == 0 {
        WIN32_ERROR(0) // Success
    } else {
        // For non-zero return, convert to WIN32_ERROR
        // Using HRESULT conversion pattern for error codes
        WIN32_ERROR(result as u32)
    }
}
