use windows::core::Result;
use windows::Win32::Foundation::WIN32_ERROR;
use windows::Win32::System::Services::{ChangeServiceConfig2A, SC_HANDLE, SERVICE_CONFIG};

#[allow(dead_code)]
fn call_change_service_config2_a() -> WIN32_ERROR {
    let result: Result<()> =
        unsafe { ChangeServiceConfig2A(SC_HANDLE(std::ptr::null_mut()), SERVICE_CONFIG(0), None) };

    match result {
        Ok(()) => WIN32_ERROR(0),
        Err(e) => WIN32_ERROR::from_error(&e).unwrap_or(WIN32_ERROR(0)),
    }
}
