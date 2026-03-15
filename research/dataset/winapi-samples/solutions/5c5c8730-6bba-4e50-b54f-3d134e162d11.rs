// TITLE: Get Primary Monitor Information

use std::mem;
use windows::{
    core::{Result, PCWSTR},
    Win32::{
        Foundation::POINT,
        Graphics::Gdi::{
            GetMonitorInfoW, MonitorFromPoint, MONITORINFO, MONITORINFOEXW,
            MONITOR_DEFAULTTOPRIMARY,
        },
        UI::WindowsAndMessaging::USER_DEFAULT_SCREEN_DPI,
    },
};

fn wchar_ptr_to_string(ptr: PCWSTR) -> String {
    // SAFETY: The pointer is valid and null-terminated as it comes from Windows API
    unsafe { ptr.to_string().unwrap() }
}

fn main() -> Result<()> {
    // Get primary monitor handle
    const ORIGIN: POINT = POINT { x: 0, y: 0 };
    let hmonitor = unsafe { MonitorFromPoint(ORIGIN, MONITOR_DEFAULTTOPRIMARY) };

    // Get monitor information
    let mut monitor_info = MONITORINFOEXW::default();
    monitor_info.monitorInfo.cbSize = mem::size_of::<MONITORINFOEXW>() as u32;

    unsafe {
        GetMonitorInfoW(
            hmonitor,
            &mut monitor_info as *mut MONITORINFOEXW as *mut MONITORINFO,
        )
    }
    .ok()?;

    // Extract monitor name and dimensions
    let device_name = wchar_ptr_to_string(PCWSTR::from_raw(monitor_info.szDevice.as_ptr()));
    let width = monitor_info.monitorInfo.rcMonitor.right - monitor_info.monitorInfo.rcMonitor.left;
    let height = monitor_info.monitorInfo.rcMonitor.bottom - monitor_info.monitorInfo.rcMonitor.top;

    println!("Primary Monitor:");
    println!("  Device: {}", device_name);
    println!("  Size: {}x{}", width, height);
    println!("  Default DPI: {}", USER_DEFAULT_SCREEN_DPI);

    Ok(())
}
