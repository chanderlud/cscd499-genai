use windows::core::HRESULT;
use windows::Win32::System::Console::{ClosePseudoConsole, HPCON};

fn call_close_pseudo_console() -> HRESULT {
    let hpc = HPCON(0);
    unsafe {
        ClosePseudoConsole(hpc);
        HRESULT(0)
    }
}
