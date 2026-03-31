use windows::Win32::UI::WindowsAndMessaging::{GetSystemMetrics, SM_CXSCREEN};

pub fn primary_screen_width_px() -> i32 {
    unsafe { GetSystemMetrics(SM_CXSCREEN) }
}
