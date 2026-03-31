#![deny(warnings)]

use windows::core::{Error, Result};
use windows::Win32::System::Search::ODBCSetTryWaitValue;

#[allow(dead_code)]
fn call_odbc_set_try_wait_value() -> Result<windows::core::BOOL> {
    // SAFETY: ODBCSetTryWaitValue is a standard Win32 API that sets a global wait value.
    let result = unsafe { ODBCSetTryWaitValue(0) };
    if result.as_bool() {
        Ok(result)
    } else {
        Err(Error::from_thread())
    }
}
