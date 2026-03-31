#![deny(warnings)]

use windows::Win32::Foundation::{GetLastError, WIN32_ERROR};
use windows::Win32::System::Search::ODBCSetTryWaitValue;

#[allow(dead_code)]
fn call_odbc_set_try_wait_value() -> WIN32_ERROR {
    let success = unsafe { ODBCSetTryWaitValue(0) };
    if success.as_bool() {
        WIN32_ERROR(0)
    } else {
        unsafe { GetLastError() }
    }
}
