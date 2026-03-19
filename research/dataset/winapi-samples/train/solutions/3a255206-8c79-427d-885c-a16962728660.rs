use windows::core::{Error, Result};
use windows::Win32::Foundation::E_FAIL;
use windows::Win32::UI::WindowsAndMessaging::SetProcessDPIAware;

fn main() -> Result<()> {
    unsafe {
        if !SetProcessDPIAware().as_bool() {
            return Err(Error::new(E_FAIL, "Failed to set process as DPI aware"));
        }
    }

    println!("Process set as DPI aware");
    Ok(())
}
