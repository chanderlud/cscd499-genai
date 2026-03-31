use windows::core::{Error, Result};
use windows::Win32::System::Environment::CallEnclave;

fn call_call_enclave() -> Result<Result<()>> {
    // SAFETY: Concrete null parameters are passed for demonstration.
    // The API call is wrapped, and any HRESULT failure is captured in the inner Result.
    let res = unsafe { CallEnclave(0, std::ptr::null(), false, std::ptr::null_mut()) };
    Ok(res)
}
