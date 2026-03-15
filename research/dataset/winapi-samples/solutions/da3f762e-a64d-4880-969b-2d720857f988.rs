// Get DPI for a window using GetDpiForWindow with fallback to GetDpiForMonitor

use windows::Win32::{
    Foundation::HWND,
    Graphics::Gdi::{
        GetDC, GetDeviceCaps, MonitorFromWindow, ReleaseDC, HMONITOR, LOGPIXELSX,
        MONITOR_DEFAULTTONEAREST,
    },
    UI::HiDpi::{GetDpiForMonitor, GetDpiForWindow, MDT_EFFECTIVE_DPI},
    UI::WindowsAndMessaging::USER_DEFAULT_SCREEN_DPI,
};

fn hwnd_dpi(hwnd: HWND) -> u32 {
    // Get device context for the window
    let hdc = unsafe { GetDC(Some(hwnd)) };
    if hdc.is_invalid() {
        panic!("`GetDC` returned null!");
    }

    // Ensure we release the DC when done
    struct DcGuard(HWND, windows::Win32::Graphics::Gdi::HDC);
    impl Drop for DcGuard {
        fn drop(&mut self) {
            unsafe {
                ReleaseDC(Some(self.0), self.1);
            }
        }
    }
    let _guard = DcGuard(hwnd, hdc);

    // Try GetDpiForWindow first (Windows 10 Anniversary Update 1607+)
    let dpi = unsafe { GetDpiForWindow(hwnd) };
    if dpi != 0 {
        return dpi;
    }

    // Fall back to GetDpiForMonitor (Windows 8.1+)
    let monitor = unsafe { MonitorFromWindow(hwnd, MONITOR_DEFAULTTONEAREST) };
    if !monitor.is_invalid() {
        let mut dpi_x = 0;
        let mut dpi_y = 0;
        if unsafe { GetDpiForMonitor(monitor, MDT_EFFECTIVE_DPI, &mut dpi_x, &mut dpi_y) }.is_ok() {
            return dpi_x;
        }
    }

    // Final fallback to device caps (Vista+)
    unsafe { GetDeviceCaps(Some(hdc), LOGPIXELSX) as u32 }
}

fn main() {
    // Example usage: Get DPI for desktop window
    let desktop_hwnd = HWND(std::ptr::null_mut()); // NULL HWND gets desktop window
    let dpi = hwnd_dpi(desktop_hwnd);
    println!("Desktop window DPI: {}", dpi);
}
