use windows::core::{Error, Result};
use windows::Win32::System::Search::ODBCGetTryWaitValue;

fn call_odbc_get_try_wait_value() -> Result<u32> {
    // SAFETY: ODBCGetTryWaitValue is a simple configuration getter that does not require special safety conditions.
    Ok(unsafe { ODBCGetTryWaitValue() })
}
