use windows::core::{Result, BOOL};
use windows::Win32::Foundation::{LPARAM, RECT};
use windows::Win32::Graphics::Gdi::{
    EnumDisplayMonitors, GetMonitorInfoW, HDC, HMONITOR, MONITORINFOEXW,
};

// Callback function for monitor enumeration
// Matches the MONITORENUMPROC signature
unsafe extern "system" fn monitor_enum_proc(
    hmonitor: HMONITOR,
    _hdc: HDC,
    _lprc: *mut RECT,
    _lparam: LPARAM,
) -> BOOL {
    // Allocate a MONITORINFOEXW structure on the heap
    let mut monitor_info = MONITORINFOEXW::default();
    monitor_info.monitorInfo.cbSize = std::mem::size_of::<MONITORINFOEXW>() as u32;

    // Get monitor information
    if GetMonitorInfoW(hmonitor, &mut monitor_info.monitorInfo) != BOOL::from(true) {
        // Get the last error if GetMonitorInfoW fails
        let error = windows::core::Error::from_thread();
        eprintln!("Failed to get monitor info: {}", error);
        return BOOL::from(true); // Continue enumeration
    }

    // Print monitor information
    println!("=== Monitor Found ===");
    println!("Monitor Handle: {:?}", hmonitor);

    // Get monitor name (if available)
    if monitor_info.szDevice[0] != 0 {
        let monitor_name = String::from_utf16_lossy(&monitor_info.szDevice);
        println!("Device Name: {}", monitor_name);
    }

    // Get monitor dimensions
    let rect = monitor_info.monitorInfo.rcMonitor;
    println!(
        "Dimensions: {}x{} at ({}, {})",
        rect.right - rect.left,
        rect.bottom - rect.top,
        rect.left,
        rect.top
    );

    // Get work area dimensions
    let work_rect = monitor_info.monitorInfo.rcWork;
    println!(
        "Work Area: {}x{} at ({}, {})",
        work_rect.right - work_rect.left,
        work_rect.bottom - work_rect.top,
        work_rect.left,
        work_rect.top
    );

    // Get monitor flags
    let flags = monitor_info.monitorInfo.dwFlags;
    println!("Flags: 0x{:X}", flags);
    if flags & 1 != 0 {
        println!("  - MONITORINFOF_PRIMARY: Primary monitor");
    }
    if flags & 2 != 0 {
        println!("  - MONITORINFOF_SECONDARY: Secondary monitor");
    }

    println!();

    // Return TRUE to continue enumeration
    BOOL::from(true)
}

fn main() -> Result<()> {
    println!("Enumerating display monitors...\n");

    // Call EnumDisplayMonitors with NULL HDC and nullptr for clip region
    let result = unsafe {
        EnumDisplayMonitors(
            Some(HDC::default()),
            None,
            Some(monitor_enum_proc),
            LPARAM::default(),
        )
    };

    // Check the result
    if result == BOOL::from(false) {
        let error = windows::core::Error::from_thread();
        eprintln!("EnumDisplayMonitors failed: {}", error);
        return Err(error);
    }

    println!("Monitor enumeration completed successfully.");
    Ok(())
}
