use windows::core::PCSTR;
use windows::Win32::Foundation::WIN32_ERROR;
use windows::Win32::System::Shutdown::AbortSystemShutdownA;

fn call_abort_system_shutdown_a() -> WIN32_ERROR {
    // SAFETY: Passing a null PCSTR targets the local machine, which is valid.
    match unsafe { AbortSystemShutdownA(PCSTR::null()) } {
        Ok(()) => WIN32_ERROR(0),
        Err(e) => WIN32_ERROR(e.code().0 as u32),
    }
}
