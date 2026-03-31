use windows::core::{Error, Result, HRESULT};
use windows::Win32::System::Services::{ChangeServiceConfig2A, SC_HANDLE, SERVICE_CONFIG};

fn call_change_service_config2_a() -> HRESULT {
    unsafe {
        match ChangeServiceConfig2A(SC_HANDLE(std::ptr::null_mut()), SERVICE_CONFIG(0), None) {
            Ok(()) => HRESULT(0),
            Err(e) => e.code(),
        }
    }
}
