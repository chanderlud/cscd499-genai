use windows::core::{Result, Error};
use windows::Win32::UI::WindowsAndMessaging::SetProcessDPIAware;

fn main() -> Result<()> {
    // SAFETY: SetProcessDPIAware is a simple Win32 API call with no complex preconditions.
    // It returns a BOOL indicating success, but we ignore the return value as in the original sample.
    unsafe {
        SetProcessDPIAware();
    }
    
    println!("Process set as DPI aware");
    Ok(())
}