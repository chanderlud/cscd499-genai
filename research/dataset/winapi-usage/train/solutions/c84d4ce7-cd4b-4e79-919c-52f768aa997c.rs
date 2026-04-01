use windows::core::HRESULT;
use windows::Win32::Foundation::S_OK;
use windows::Win32::Graphics::Gdi::HMONITOR;
use windows::Win32::UI::HiDpi::{GetDpiForMonitor, MDT_EFFECTIVE_DPI};

fn call_get_dpi_for_monitor() -> HRESULT {
    let mut dpix: u32 = 0;
    let mut dpiy: u32 = 0;

    unsafe {
        match GetDpiForMonitor(HMONITOR::default(), MDT_EFFECTIVE_DPI, &mut dpix, &mut dpiy) {
            Ok(_) => S_OK,
            Err(e) => e.code(),
        }
    }
}
