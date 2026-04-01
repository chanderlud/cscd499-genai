use windows::Win32::Foundation::{ERROR_SUCCESS, WIN32_ERROR};
use windows::Win32::System::Services::{CloseServiceHandle, SC_HANDLE};

fn call_close_service_handle() -> WIN32_ERROR {
    let handle = SC_HANDLE(std::ptr::null_mut());

    unsafe {
        match CloseServiceHandle(handle) {
            Ok(()) => ERROR_SUCCESS,
            Err(e) => WIN32_ERROR::from_error(&e).unwrap_or(ERROR_SUCCESS),
        }
    }
}
