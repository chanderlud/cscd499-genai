use windows::core::{BOOL, HRESULT};
use windows::Win32::Foundation::POINT;
use windows::Win32::Graphics::Gdi::MonitorFromPoint;
use windows::Win32::Graphics::Gdi::{
    GetMonitorInfoW, HMONITOR, MONITORINFO, MONITORINFOEXW, MONITOR_DEFAULTTOPRIMARY,
};

fn main() -> windows::core::Result<()> {
    // Step 1: Obtain a monitor handle (using the primary monitor for simplicity)
    let point = POINT { x: 0, y: 0 };
    let hmonitor: HMONITOR = unsafe { MonitorFromPoint(point, MONITOR_DEFAULTTOPRIMARY) };

    // Step 2: Prepare MONITORINFOEXW with correct cbSize
    let mut monitor_info = MONITORINFOEXW {
        monitorInfo: MONITORINFO {
            cbSize: std::mem::size_of::<MONITORINFO>() as u32,
            ..Default::default()
        },
        ..Default::default()
    };

    // Step 3: Call GetMonitorInfoW
    let success: BOOL = unsafe { GetMonitorInfoW(hmonitor, &mut monitor_info.monitorInfo) };
    if !success.as_bool() {
        return Err(windows::core::Error::from_hresult(HRESULT::from_win32(
            unsafe { windows::Win32::Foundation::GetLastError().0 },
        )));
    }

    // Step 4: Extract monitor name from szDevice
    let monitor_name = String::from_utf16_lossy(&monitor_info.szDevice);

    // Example usage: print monitor name
    println!("Monitor name: {}", monitor_name);

    Ok(())
}
