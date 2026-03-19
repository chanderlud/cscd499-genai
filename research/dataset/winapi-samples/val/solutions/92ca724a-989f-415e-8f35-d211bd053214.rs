use windows::core::{Error, Result};
use windows::Win32::Foundation::{E_FAIL, HWND};
use windows::Win32::Graphics::Gdi::{
    GetDC, GetDeviceCaps, MonitorFromWindow, ReleaseDC, LOGPIXELSX, MONITOR_DEFAULTTONEAREST,
};
use windows::Win32::UI::HiDpi::{GetDpiForMonitor, GetDpiForWindow, MDT_EFFECTIVE_DPI};

fn hwnd_dpi(hwnd: HWND) -> Result<u32> {
    // Define fallback chain as iterator of closures
    let fallbacks: Vec<Box<dyn FnOnce() -> Option<u32>>> = vec![
        // Fallback 1: Try GetDpiForWindow (Windows 10 1607+)
        Box::new(|| {
            let dpi = unsafe { GetDpiForWindow(hwnd) };
            if dpi == 0 {
                None
            } else {
                Some(dpi)
            }
        }),
        // Fallback 2: Try GetDpiForMonitor (Windows 8.1+)
        Box::new(|| {
            // Get monitor for window
            let monitor = unsafe { MonitorFromWindow(hwnd, MONITOR_DEFAULTTONEAREST) };
            if monitor.is_invalid() {
                return None;
            }

            let mut dpi_x = 0;
            let mut dpi_y = 0;
            let result =
                unsafe { GetDpiForMonitor(monitor, MDT_EFFECTIVE_DPI, &mut dpi_x, &mut dpi_y) };

            if result.is_ok() {
                Some(dpi_x)
            } else {
                None
            }
        }),
        // Fallback 3: Try GetDeviceCaps with LOGPIXELSX
        Box::new(|| {
            // Get device context - NULL HWND gets desktop DC
            let hdc = unsafe { GetDC(Some(hwnd)) };
            if hdc.is_invalid() {
                return None;
            }

            // Get horizontal DPI
            let dpi = unsafe { GetDeviceCaps(Some(hdc), LOGPIXELSX) };

            // Always release the DC
            unsafe { ReleaseDC(Some(hwnd), hdc) };

            if dpi == 0 {
                None
            } else {
                Some(dpi as u32)
            }
        }),
    ];

    // Execute fallback chain
    for fallback in fallbacks {
        if let Some(dpi) = fallback() {
            return Ok(dpi);
        }
    }

    // All fallbacks failed
    Err(Error::from_hresult(E_FAIL))
}

fn main() -> Result<()> {
    let hwnd = None;
    let dpi = hwnd_dpi(hwnd)?;
    println!("DPI: {}", dpi);
    Ok(())
}
