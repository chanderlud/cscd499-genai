use windows::Win32::Foundation::HWND;
use windows::Win32::UI::WindowsAndMessaging::{
    GetDesktopWindow, GetWindowLongPtrW, GWL_WNDPROC, WINDOW_LONG_PTR_INDEX,
};

#[cfg(target_pointer_width = "32")]
use windows::Win32::UI::WindowsAndMessaging::GetWindowLongW;

#[cfg(target_pointer_width = "32")]
fn get_window_long_ptr_w(window: HWND, index: WINDOW_LONG_PTR_INDEX) -> isize {
    // SAFETY: Caller must ensure window handle is valid and index is appropriate.
    // Convert WINDOW_LONG_PTR_INDEX to i32 for GetWindowLongW
    unsafe { GetWindowLongW(window, index.0) as _ }
}

#[cfg(target_pointer_width = "64")]
fn get_window_long_ptr_w(window: HWND, index: WINDOW_LONG_PTR_INDEX) -> isize {
    // SAFETY: Caller must ensure window handle is valid and index is appropriate.
    unsafe { GetWindowLongPtrW(window, index) }
}

fn main() -> windows::core::Result<()> {
    let hwnd = unsafe { GetDesktopWindow() };
    let wnd_proc = get_window_long_ptr_w(hwnd, GWL_WNDPROC);
    println!("Window procedure address: {:#x}", wnd_proc);
    Ok(())
}
