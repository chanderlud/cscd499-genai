use windows::core::{Error, Result};
use windows::Win32::Foundation::{ERROR_SUCCESS, WIN32_ERROR};
use windows::Win32::Graphics::Gdi::HMONITOR;
use windows::Win32::UI::HiDpi::{GetDpiForMonitor, MDT_EFFECTIVE_DPI};

fn call_get_dpi_for_monitor() -> Result<WIN32_ERROR> {
    let mut dpix: u32 = 0;
    let mut dpiy: u32 = 0;

    // Use default monitor (primary monitor)
    let hmonitor = HMONITOR::default();

    // Use effective DPI type
    let dpitype = MDT_EFFECTIVE_DPI;

    // Call GetDpiForMonitor
    unsafe { GetDpiForMonitor(hmonitor, dpitype, &mut dpix, &mut dpiy)? };

    Ok(ERROR_SUCCESS)
}
