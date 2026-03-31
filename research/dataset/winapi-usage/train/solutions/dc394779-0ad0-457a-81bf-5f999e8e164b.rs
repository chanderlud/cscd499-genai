use windows::core::{Error, Result, HRESULT};
use windows::Win32::System::Power::{CallNtPowerInformation, POWER_INFORMATION_LEVEL};

fn call_call_nt_power_information() -> HRESULT {
    // SAFETY: Passing null buffers and zero lengths is valid and safe for this API call.
    let status = unsafe { CallNtPowerInformation(POWER_INFORMATION_LEVEL(0), None, 0, None, 0) };
    HRESULT(status.0)
}
