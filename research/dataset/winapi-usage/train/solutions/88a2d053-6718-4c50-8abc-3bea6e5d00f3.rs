use windows::core::{Error, Result};
use windows::Win32::System::Console::{ClosePseudoConsole, HPCON};

fn call_close_pseudo_console() -> Result<()> {
    // SAFETY: Calling ClosePseudoConsole with a concrete dummy handle value.
    // In production, this would be a valid HPCON obtained from CreatePseudoConsole.
    unsafe {
        ClosePseudoConsole(HPCON(0));
    }
    Ok(())
}
