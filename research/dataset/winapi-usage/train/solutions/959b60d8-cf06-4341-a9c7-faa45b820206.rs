use windows::core::{Error, Result, HRESULT};
use windows::Win32::System::Services::{ChangeServiceConfig2W, SC_HANDLE, SERVICE_CONFIG};

fn call_change_service_config2_w() -> HRESULT {
    // SAFETY: Passing null/zero values is safe for this API call demonstration;
    // the API will fail gracefully with an error code which we capture and return.
    unsafe {
        ChangeServiceConfig2W(SC_HANDLE(std::ptr::null_mut()), SERVICE_CONFIG(0), None)
            .map(|_| HRESULT(0))
            .unwrap_or_else(|e| e.code())
    }
}
