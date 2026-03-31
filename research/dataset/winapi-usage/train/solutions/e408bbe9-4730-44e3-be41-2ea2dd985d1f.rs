use windows::core::{w, Error, Result, HRESULT};
use windows::Win32::System::Shutdown::AbortSystemShutdownW;

fn call_abort_system_shutdown_w() -> windows::core::HRESULT {
    // AbortSystemShutdownW is an unsafe FFI call that requires a valid PCWSTR.
    unsafe { AbortSystemShutdownW(w!("")) }
        .map(|_| HRESULT(0))
        .unwrap_or_else(|e| e.code())
}
