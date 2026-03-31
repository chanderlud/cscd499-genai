use windows::core::{Error, Result};
use windows::Win32::Foundation::{GetLastError, HANDLE, WIN32_ERROR};
use windows::Win32::System::Environment::CreateEnclave;

fn call_create_enclave() -> WIN32_ERROR {
    let result =
        unsafe { CreateEnclave(HANDLE::default(), None, 0, 0, 0, std::ptr::null(), 0, None) };

    if result.is_null() {
        unsafe { GetLastError() }
    } else {
        WIN32_ERROR(0)
    }
}
