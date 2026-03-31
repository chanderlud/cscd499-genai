use windows::core::Error;
use windows::Win32::Foundation::{HWND, LPARAM, LRESULT, WPARAM};
use windows::Win32::Graphics::Dwm::DwmDefWindowProc;

fn call_dwm_def_window_proc() -> windows::core::HRESULT {
    let mut lresult = LRESULT(0);
    let success =
        unsafe { DwmDefWindowProc(HWND::default(), 0, WPARAM(0), LPARAM(0), &mut lresult) };

    if success.as_bool() {
        windows::core::HRESULT(0)
    } else {
        Error::from_thread().code()
    }
}
