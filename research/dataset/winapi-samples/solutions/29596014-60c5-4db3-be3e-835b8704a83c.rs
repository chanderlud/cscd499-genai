// TITLE: GDI Monitor Device Name from HMONITOR

use std::ffi::OsString;
use std::os::windows::ffi::OsStringExt;

use windows::core::Result;
use windows::Win32::Foundation::HWND;
use windows::Win32::Graphics::Gdi::{
    GetMonitorInfoW, MonitorFromWindow, HMONITOR, MONITORINFOEXW, MONITOR_DEFAULTTOPRIMARY,
};

fn wide_to_string(wide: &[u16]) -> String {
    let len = wide.iter().position(|&c| c == 0).unwrap_or(wide.len());
    let os_string = OsString::from_wide(&wide[..len]);
    os_string.to_string_lossy().into_owned()
}

fn get_monitor_device_name(hmonitor: HMONITOR) -> Result<String> {
    let mut monitor_info = MONITORINFOEXW::default();
    monitor_info.monitorInfo.cbSize = std::mem::size_of::<MONITORINFOEXW>() as u32;

    // SAFETY: GetMonitorInfoW writes to our MONITORINFOEXW struct
    unsafe {
        GetMonitorInfoW(hmonitor, &mut monitor_info.monitorInfo).ok()?;
    }

    Ok(wide_to_string(&monitor_info.szDevice))
}

fn main() -> Result<()> {
    // Get primary monitor handle
    // SAFETY: Passing null HWND and MONITOR_DEFAULTTOPRIMARY is safe
    let hmonitor = unsafe { MonitorFromWindow(HWND::default(), MONITOR_DEFAULTTOPRIMARY) };

    let device_name = get_monitor_device_name(hmonitor)?;
    println!("Primary monitor device name: {}", device_name);

    Ok(())
}
