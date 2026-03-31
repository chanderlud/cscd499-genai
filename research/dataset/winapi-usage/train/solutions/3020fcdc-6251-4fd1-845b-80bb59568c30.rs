use windows::core::{Error, Result};
use windows::Win32::System::Services::{ChangeServiceConfig2W, SC_HANDLE, SERVICE_CONFIG};

fn call_change_service_config2_w() -> Result<Result<()>> {
    let hservice = SC_HANDLE::default();
    let dwinfolevel = SERVICE_CONFIG(0);
    let lpinfo = None;

    // SAFETY: Calling ChangeServiceConfig2W with null/default parameters is safe for demonstration.
    // The API will return an error code which is captured by the Result type.
    let api_result = unsafe { ChangeServiceConfig2W(hservice, dwinfolevel, lpinfo) };
    Ok(api_result)
}
