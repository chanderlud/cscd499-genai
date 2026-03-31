use windows::core::{Error, Result};
use windows::Win32::Foundation::WIN32_ERROR;
use windows::Win32::System::Search::ODBCGetTryWaitValue;

fn call_odbc_get_try_wait_value() -> WIN32_ERROR {
    // SAFETY: ODBCGetTryWaitValue is a simple getter with no preconditions or side effects.
    unsafe { WIN32_ERROR(ODBCGetTryWaitValue()) }
}
