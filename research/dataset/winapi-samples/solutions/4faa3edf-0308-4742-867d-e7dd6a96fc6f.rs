use windows::core::Result;
use windows::Win32::Foundation::HWND;
use windows::Win32::Graphics::Gdi::{
    GetMonitorInfoW, MonitorFromWindow, MONITORINFO, MONITOR_DEFAULTTONEAREST,
};
use windows::Win32::UI::WindowsAndMessaging::SetProcessDPIAware;

fn main() -> Result<()> {
    // Set process DPI aware
    unsafe {
        SetProcessDPIAware().ok()?;
    }

    // Get monitor info for the desktop window
    let desktop_window = HWND::default();
    let monitor = unsafe { MonitorFromWindow(desktop_window, MONITOR_DEFAULTTONEAREST) };

    let mut monitor_info = MONITORINFO {
        cbSize: std::mem::size_of::<MONITORINFO>() as u32,
        ..Default::default()
    };

    unsafe {
        GetMonitorInfoW(monitor, &mut monitor_info).ok()?;
    }

    println!(
        "Monitor bounds: ({}, {}) to ({}, {})",
        monitor_info.rcMonitor.left,
        monitor_info.rcMonitor.top,
        monitor_info.rcMonitor.right,
        monitor_info.rcMonitor.bottom
    );

    Ok(())
}
