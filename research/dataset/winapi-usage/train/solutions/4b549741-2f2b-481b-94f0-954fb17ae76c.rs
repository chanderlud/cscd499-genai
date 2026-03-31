use windows::core::w;
use windows::Win32::Foundation::WIN32_ERROR;
use windows::Win32::System::Shutdown::AbortSystemShutdownW;

fn call_abort_system_shutdown_w() -> WIN32_ERROR {
    match unsafe { AbortSystemShutdownW(w!("")) } {
        Ok(()) => WIN32_ERROR(0),
        Err(e) => WIN32_ERROR::from_error(&e).unwrap_or_default(),
    }
}
