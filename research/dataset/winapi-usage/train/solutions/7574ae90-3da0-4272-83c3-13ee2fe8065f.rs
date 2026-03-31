use windows::Win32::Foundation::{GetLastError, HWND, LPARAM, LRESULT, WIN32_ERROR, WPARAM};
use windows::Win32::Graphics::Dwm::DwmDefWindowProc;

fn call_dwm_def_window_proc() -> WIN32_ERROR {
    unsafe {
        let mut lresult = LRESULT(0);
        let hwnd = HWND(std::ptr::null_mut());
        let success = DwmDefWindowProc(hwnd, 0, WPARAM(0), LPARAM(0), &mut lresult);

        if success.as_bool() {
            WIN32_ERROR(0)
        } else {
            GetLastError()
        }
    }
}
