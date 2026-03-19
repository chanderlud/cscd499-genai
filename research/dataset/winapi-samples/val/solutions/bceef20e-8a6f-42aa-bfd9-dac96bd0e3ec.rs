use windows::core::Result;
use windows::Win32::Graphics::Gdi::{
    GetMonitorInfoW, MonitorFromWindow, MONITORINFO, MONITOR_DEFAULTTONEAREST,
};
use windows::Win32::UI::WindowsAndMessaging::{GetDesktopWindow, SetProcessDPIAware};

fn main() -> Result<()> {
    // Set the process to be DPI aware
    unsafe {
        SetProcessDPIAware().ok()?;
    }

    // Get the desktop window handle
    let desktop_window = unsafe { GetDesktopWindow() };

    // Get the monitor that the desktop window is mostly on
    let monitor = unsafe { MonitorFromWindow(desktop_window, MONITOR_DEFAULTTONEAREST) };

    // Get monitor information
    let mut monitor_info = MONITORINFO {
        cbSize: std::mem::size_of::<MONITORINFO>() as u32,
        ..Default::default()
    };

    unsafe {
        GetMonitorInfoW(monitor, &mut monitor_info).ok()?;
    }

    println!("Desktop monitor bounds:");
    println!("  Left: {}", monitor_info.rcMonitor.left);
    println!("  Top: {}", monitor_info.rcMonitor.top);
    println!("  Right: {}", monitor_info.rcMonitor.right);
    println!("  Bottom: {}", monitor_info.rcMonitor.bottom);
    println!(
        "  Width: {}",
        monitor_info.rcMonitor.right - monitor_info.rcMonitor.left
    );
    println!(
        "  Height: {}",
        monitor_info.rcMonitor.bottom - monitor_info.rcMonitor.top
    );

    Ok(())
}
