use std::mem;
use windows::{
    core::{Error, Result, PCWSTR},
    Win32::{
        Foundation::POINT,
        Graphics::Gdi::{
            GetMonitorInfoW, MonitorFromPoint, HMONITOR, MONITORINFO, MONITORINFOEXW,
            MONITOR_DEFAULTTONULL,
        },
    },
};

fn wchar_ptr_to_string(ptr: PCWSTR) -> String {
    unsafe {
        let len = (0..).take_while(|&i| *ptr.0.offset(i) != 0).count();
        let slice = std::slice::from_raw_parts(ptr.0, len);
        String::from_utf16_lossy(slice)
    }
}

fn get_monitor_info(hmonitor: HMONITOR) -> Result<MONITORINFOEXW> {
    let mut monitor_info = MONITORINFOEXW::default();
    monitor_info.monitorInfo.cbSize = mem::size_of::<MONITORINFOEXW>() as u32;
    let status = unsafe {
        GetMonitorInfoW(
            hmonitor,
            &mut monitor_info as *mut MONITORINFOEXW as *mut MONITORINFO,
        )
    };
    if !status.as_bool() {
        Err(Error::from_thread())
    } else {
        Ok(monitor_info)
    }
}

fn main() -> Result<()> {
    let point = POINT { x: 100, y: 100 };
    let hmonitor = unsafe { MonitorFromPoint(point, MONITOR_DEFAULTTONULL) };

    if hmonitor.is_invalid() {
        println!("No monitor found at point (100, 100)");
        return Ok(());
    }

    let monitor_info = get_monitor_info(hmonitor)?;
    let device_name = wchar_ptr_to_string(PCWSTR::from_raw(monitor_info.szDevice.as_ptr()));
    let rect = monitor_info.monitorInfo.rcMonitor;

    println!("Monitor device name: {}", device_name);
    println!(
        "Monitor rectangle: left={}, top={}, right={}, bottom={}",
        rect.left, rect.top, rect.right, rect.bottom
    );

    Ok(())
}
