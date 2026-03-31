use windows::Win32::Foundation::WIN32_ERROR;
use windows::Win32::System::Services::{ChangeServiceConfig2W, SC_HANDLE, SERVICE_CONFIG};

fn call_change_service_config2_w() -> windows::Win32::Foundation::WIN32_ERROR {
    let result =
        unsafe { ChangeServiceConfig2W(SC_HANDLE(std::ptr::null_mut()), SERVICE_CONFIG(0), None) };
    match result {
        Ok(()) => WIN32_ERROR(0),
        Err(e) => WIN32_ERROR::from_error(&e).unwrap_or_default(),
    }
}
