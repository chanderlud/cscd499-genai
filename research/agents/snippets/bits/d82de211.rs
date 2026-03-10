use windows::core::Result;
use windows::Win32::UI::WindowsAndMessaging::{GetSystemMetrics, SM_CXSCREEN, SM_CYSCREEN};

fn main() -> Result<()> {
    // Retrieve width and height in a single unsafe block for clarity
    let (width, height) = unsafe {
        (
            GetSystemMetrics(SM_CXSCREEN),
            GetSystemMetrics(SM_CYSCREEN),
        )
    };

    // Validate results using pattern matching to avoid manual comparison with negative values
    match (width, height) {
        (w, h) if w >= 0 && h >= 0 => println!("Display resolution: {} x {}", w, h),
        _ => return Err(windows::core::Error::from_thread()),
    }

    Ok(())
}
