use windows::core::HRESULT;
use windows::core::{Error, Result};
use windows::Win32::System::Environment::CallEnclave;

fn call_call_enclave() -> HRESULT {
    unsafe {
        CallEnclave(0, std::ptr::null(), false, std::ptr::null_mut())
            .map(|_| HRESULT(0))
            .unwrap_or_else(|e| e.code())
    }
}
