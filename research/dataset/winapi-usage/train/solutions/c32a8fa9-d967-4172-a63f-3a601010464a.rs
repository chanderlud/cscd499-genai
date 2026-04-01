use windows::Win32::Foundation::{HWND, LPARAM, LRESULT, WIN32_ERROR, WPARAM};
use windows::Win32::UI::WindowsAndMessaging::{CallWindowProcW, WNDPROC};

unsafe extern "system" fn dummy_window_proc(
    hwnd: HWND,
    msg: u32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    LRESULT(0)
}

fn call_call_window_proc_w() -> WIN32_ERROR {
    let proc: WNDPROC = Some(dummy_window_proc);

    unsafe {
        CallWindowProcW(proc, HWND::default(), 0x0001, WPARAM(0), LPARAM(0));
    }

    WIN32_ERROR(0)
}
