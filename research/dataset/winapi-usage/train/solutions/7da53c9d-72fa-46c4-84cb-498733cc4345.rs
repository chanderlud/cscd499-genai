use windows::core::{Error, Result};
use windows::Win32::Foundation::WIN32_ERROR;
use windows::Win32::NetworkManagement::NetManagement::LogErrorA;

fn call_log_error_a() -> WIN32_ERROR {
    // LogErrorA is a void Win32 API, so it returns () in the windows crate.
    // We call it with concrete dummy values and return a success code.
    unsafe {
        LogErrorA(0, &[], 0);
    }
    WIN32_ERROR(0)
}
