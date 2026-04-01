use windows::core::HRESULT;
use windows::core::{Error, Result};
use windows::Win32::Foundation::{HWND, LPARAM, LRESULT, WPARAM};
use windows::Win32::UI::WindowsAndMessaging::{CallWindowProcW, WNDPROC};

unsafe extern "system" fn my_wndproc(
    hwnd: HWND,
    msg: u32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    LRESULT(0)
}

fn call_call_window_proc_w() -> HRESULT {
    let wndproc: WNDPROC = Some(my_wndproc);

    let result: LRESULT =
        unsafe { CallWindowProcW(wndproc, HWND::default(), 0, WPARAM(0), LPARAM(0)) };

    HRESULT(result.0 as i32)
}
