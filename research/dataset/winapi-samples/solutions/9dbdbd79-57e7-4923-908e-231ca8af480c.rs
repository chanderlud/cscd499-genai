// Get monitor information for a window using MonitorFromWindow and GetMonitorInfoW

use windows::core::{Error, Result};
use windows::Win32::Foundation::{HWND, RECT};
use windows::Win32::Graphics::Gdi::{
    GetMonitorInfoW, MonitorFromWindow, MONITORINFO, MONITOR_DEFAULTTONEAREST,
};
use windows::Win32::UI::WindowsAndMessaging::GetDesktopWindow;

fn main() -> Result<()> {
    // Get the desktop window handle
    let desktop_window = unsafe { GetDesktopWindow() };

    // Get the monitor that the desktop window is mostly on
    let monitor = unsafe { MonitorFromWindow(desktop_window, MONITOR_DEFAULTTONEAREST) };

    // Prepare MONITORINFO structure
    let mut monitor_info = MONITORINFO {
        cbSize: std::mem::size_of::<MONITORINFO>() as u32,
        ..Default::default()
    };

    // Get monitor information
    let success = unsafe { GetMonitorInfoW(monitor, &mut monitor_info) };
    if !success.as_bool() {
        return Err(Error::from_thread());
    }

    // Print monitor bounds
    println!("Monitor bounds:");
    println!("  Left: {}", monitor_info.rcMonitor.left);
    println!("  Top: {}", monitor_info.rcMonitor.top);
    println!("  Right: {}", monitor_info.rcMonitor.right);
    println!("  Bottom: {}", monitor_info.rcMonitor.bottom);

    // Print work area (excludes taskbar)
    println!("\nWork area:");
    println!("  Left: {}", monitor_info.rcWork.left);
    println!("  Top: {}", monitor_info.rcWork.top);
    println!("  Right: {}", monitor_info.rcWork.right);
    println!("  Bottom: {}", monitor_info.rcWork.bottom);

    Ok(())
}
