use windows::core::HRESULT;
use windows::core::{Error, Result};
use windows::Win32::Foundation::ERROR_GEN_FAILURE;
use windows::Win32::System::Search::ODBCSetTryWaitValue;

fn call_odbc_set_try_wait_value() -> HRESULT {
    let result = unsafe { ODBCSetTryWaitValue(0) };
    if result.as_bool() {
        HRESULT(0)
    } else {
        HRESULT::from_win32(ERROR_GEN_FAILURE.0)
    }
}
