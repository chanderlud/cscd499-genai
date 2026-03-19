use windows::Win32::{
    Foundation::HWND,
    Graphics::Gdi::{
        GetDC, GetDeviceCaps, MonitorFromWindow, ReleaseDC, LOGPIXELSX, MONITOR_DEFAULTTONEAREST,
    },
    UI::HiDpi::{GetDpiForMonitor, GetDpiForWindow, MDT_EFFECTIVE_DPI},
};

fn hwnd_dpi(hwnd: HWND) -> u32 {
    let hdc = unsafe { GetDC(Some(hwnd)) };
    if hdc.is_invalid() {
        panic!("`GetDC` returned null!");
    }

    struct DcGuard(HWND, windows::Win32::Graphics::Gdi::HDC);
    impl Drop for DcGuard {
        fn drop(&mut self) {
            unsafe {
                ReleaseDC(Some(self.0), self.1);
            }
        }
    }
    let _guard = DcGuard(hwnd, hdc);

    let dpi = unsafe { GetDpiForWindow(hwnd) };
    if dpi != 0 {
        return dpi;
    }

    let monitor = unsafe { MonitorFromWindow(hwnd, MONITOR_DEFAULTTONEAREST) };
    if !monitor.is_invalid() {
        let mut dpi_x = 0;
        let mut dpi_y = 0;
        if unsafe { GetDpiForMonitor(monitor, MDT_EFFECTIVE_DPI, &mut dpi_x, &mut dpi_y) }.is_ok() {
            return dpi_x;
        }
    }

    unsafe { GetDeviceCaps(Some(hdc), LOGPIXELSX) as u32 }
}

fn main() {
    let desktop_hwnd = HWND(std::ptr::null_mut());
    let dpi = hwnd_dpi(desktop_hwnd);
    println!("Desktop window DPI: {}", dpi);
}
