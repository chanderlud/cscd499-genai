use windows::core::{Error, Result, HRESULT};
use windows::Win32::System::Search::ODBCGetTryWaitValue;

fn call_odbc_get_try_wait_value() -> HRESULT {
    // SAFETY: ODBCGetTryWaitValue is a simple getter with no preconditions or side effects.
    let value = unsafe { ODBCGetTryWaitValue() };
    HRESULT::from_win32(value)
}
