// Get desktop window rectangle with error handling

use windows::core::Result;
use windows::Win32::Foundation::RECT;
use windows::Win32::UI::WindowsAndMessaging::{GetDesktopWindow, GetWindowRect};

fn get_desktop_window_rect() -> Result<RECT> {
    let hwnd = unsafe { GetDesktopWindow() };
    let mut rect = RECT::default();
    unsafe { GetWindowRect(hwnd, &mut rect)? };
    Ok(rect)
}

fn main() -> Result<()> {
    let rect = get_desktop_window_rect()?;
    println!(
        "Desktop window rect: left={}, top={}, right={}, bottom={}",
        rect.left, rect.top, rect.right, rect.bottom
    );
    Ok(())
}
