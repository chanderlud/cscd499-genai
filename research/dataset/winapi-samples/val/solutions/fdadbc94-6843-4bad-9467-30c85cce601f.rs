// TITLE: Check if the active window is maximized using GetWindowPlacement

use windows::core::Result;
use windows::Win32::Foundation::HWND;
use windows::Win32::UI::Input::KeyboardAndMouse::GetActiveWindow;
use windows::Win32::UI::WindowsAndMessaging::{GetWindowPlacement, SW_MAXIMIZE, WINDOWPLACEMENT};

fn is_maximized(window: HWND) -> Result<bool> {
    let mut placement = WINDOWPLACEMENT {
        length: std::mem::size_of::<WINDOWPLACEMENT>() as u32,
        ..WINDOWPLACEMENT::default()
    };
    unsafe { GetWindowPlacement(window, &mut placement)? };
    Ok(placement.showCmd == SW_MAXIMIZE.0 as u32)
}

fn main() -> Result<()> {
    let window = unsafe { GetActiveWindow() };
    if window.is_invalid() {
        println!("No active window");
        return Ok(());
    }
    let maximized = is_maximized(window)?;
    println!("Active window is maximized: {}", maximized);
    Ok(())
}
