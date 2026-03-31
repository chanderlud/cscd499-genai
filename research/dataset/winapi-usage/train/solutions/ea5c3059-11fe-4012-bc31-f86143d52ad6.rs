use windows::core::{Error, Result};
use windows::Win32::System::Services::{ChangeServiceConfig2A, SC_HANDLE, SERVICE_CONFIG};

fn call_change_service_config2_a() -> Result<()> {
    // SAFETY: Passing a null handle and default config level is safe for API binding verification.
    // The Win32 API will fail gracefully, and the error is propagated via the Result type.
    unsafe { ChangeServiceConfig2A(SC_HANDLE(std::ptr::null_mut()), SERVICE_CONFIG(0), None) }
}
