use windows::core::{Error, Result};
use windows::Win32::Foundation::HWND;
use windows::Win32::Graphics::Gdi::{
    GetMonitorInfoW, MonitorFromWindow, MONITORINFO, MONITOR_DEFAULTTONEAREST,
};
use windows::Win32::UI::WindowsAndMessaging::SetProcessDPIAware;

fn main() -> Result<()> {
    // SAFETY: Calling SetProcessDPIAware is safe as long as we are in a Windows GUI thread.
    unsafe {
        SetProcessDPIAware();
    }

    // Get primary monitor handle
    // SAFETY: Passing NULL window handle with MONITOR_DEFAULTTONEAREST is safe
    let hmonitor = unsafe { MonitorFromWindow(HWND::default(), MONITOR_DEFAULTTONEAREST) };

    let mut monitor_info = MONITORINFO {
        cbSize: std::mem::size_of::<MONITORINFO>() as u32,
        ..Default::default()
    };

    // SAFETY: We are initializing the MONITORINFO structure correctly and passing it to GetMonitorInfoW
    let success = unsafe { GetMonitorInfoW(hmonitor, &mut monitor_info) };
    if !success.as_bool() {
        // GetMonitorInfoW failed, capture the error from GetLastError
        return Err(Error::from_thread());
    }

    println!("Primary monitor rectangle:");
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
