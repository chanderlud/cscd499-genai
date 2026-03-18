use windows::core::{Error, Result};
use windows::Win32::Foundation::RECT;
use windows::Win32::Graphics::Gdi::{
    GetMonitorInfoW, MonitorFromRect, MONITORINFO, MONITOR_DEFAULTTOPRIMARY,
};
use windows::Win32::UI::WindowsAndMessaging::{
    GetSystemMetrics, SetProcessDPIAware, SM_CXVIRTUALSCREEN, SM_CYVIRTUALSCREEN,
    SM_XVIRTUALSCREEN, SM_YVIRTUALSCREEN,
};

fn get_desktop_monitor_bounds() -> Result<RECT> {
    // Set process DPI aware
    unsafe {
        if !SetProcessDPIAware().as_bool() {
            return Err(Error::from_thread());
        }
    }

    // Get virtual screen rectangle
    let virtual_rect = unsafe {
        RECT {
            left: GetSystemMetrics(SM_XVIRTUALSCREEN),
            top: GetSystemMetrics(SM_YVIRTUALSCREEN),
            right: GetSystemMetrics(SM_XVIRTUALSCREEN) + GetSystemMetrics(SM_CXVIRTUALSCREEN),
            bottom: GetSystemMetrics(SM_YVIRTUALSCREEN) + GetSystemMetrics(SM_CYVIRTUALSCREEN),
        }
    };

    // Get monitor from virtual rectangle, defaulting to primary if empty
    let monitor = unsafe { MonitorFromRect(&virtual_rect, MONITOR_DEFAULTTOPRIMARY) };

    // Get monitor info
    let mut monitor_info = MONITORINFO {
        cbSize: std::mem::size_of::<MONITORINFO>() as u32,
        rcMonitor: RECT::default(),
        rcWork: RECT::default(),
        dwFlags: 0,
    };

    unsafe {
        if !GetMonitorInfoW(monitor, &mut monitor_info).as_bool() {
            return Err(Error::from_thread());
        }
    }

    Ok(monitor_info.rcMonitor)
}
