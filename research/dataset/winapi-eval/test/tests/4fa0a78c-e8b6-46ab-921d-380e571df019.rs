#[cfg(test)]
mod tests {
    use super::*;
    use windows::Win32::UI::WindowsAndMessaging::{GetSystemMetrics, SM_CXSCREEN};

    #[test]
    fn returns_positive_width() {
        let width = primary_screen_width_px();
        assert!(width > 0);
    }

    #[test]
    fn matches_direct_win32_call() {
        let wrapped = primary_screen_width_px();
        let direct = unsafe { GetSystemMetrics(SM_CXSCREEN) };
        assert_eq!(wrapped, direct);
    }
}
