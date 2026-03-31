use windows::Win32::Foundation::{GetLastError, WIN32_ERROR};
use windows::Win32::System::Console::{ClosePseudoConsole, HPCON};

fn call_close_pseudo_console() -> WIN32_ERROR {
    unsafe {
        ClosePseudoConsole(HPCON(0));
        GetLastError()
    }
}
