#![deny(warnings)]

use windows::core::{HRESULT, PCSTR};
use windows::Win32::System::Shutdown::AbortSystemShutdownA;

#[allow(dead_code)]
fn call_abort_system_shutdown_a() -> HRESULT {
    unsafe {
        match AbortSystemShutdownA(PCSTR::null()) {
            Ok(()) => HRESULT(0),
            Err(e) => e.code(),
        }
    }
}
