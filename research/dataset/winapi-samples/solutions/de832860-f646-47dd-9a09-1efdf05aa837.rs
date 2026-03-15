use windows::core::{Error, Result, HRESULT};
use windows::Win32::UI::WindowsAndMessaging::{GetSystemMetrics, SM_CXSCREEN, SM_CYSCREEN};

fn get_primary_screen_size() -> Result<(i32, i32)> {
    unsafe {
        let width = GetSystemMetrics(SM_CXSCREEN);
        let height = GetSystemMetrics(SM_CYSCREEN);

        if width == 0 || height == 0 {
            // GetSystemMetrics doesn't set GetLastError on failure, so use a generic error
            Err(Error::from_hresult(HRESULT::from_win32(0)))
        } else {
            Ok((width, height))
        }
    }
}

fn main() -> Result<()> {
    let (width, height) = get_primary_screen_size()?;
    println!("Primary screen size: {}x{}", width, height);
    Ok(())
}
