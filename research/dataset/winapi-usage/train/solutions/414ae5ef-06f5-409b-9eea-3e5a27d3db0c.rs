#[allow(unused_imports)]
use windows::core::{Error, Result, HRESULT};
use windows::Win32::Foundation::HWND;
use windows::Win32::Graphics::Gdi::HDC;
use windows::Win32::UI::Controls::{BeginBufferedAnimation, BP_BUFFERFORMAT};

fn call_begin_buffered_animation() -> HRESULT {
    unsafe {
        let result = BeginBufferedAnimation(
            HWND::default(),
            HDC::default(),
            std::ptr::null(),
            BP_BUFFERFORMAT(0),
            None,
            std::ptr::null(),
            std::ptr::null_mut(),
            std::ptr::null_mut(),
        );
        if result == 0 {
            Error::from_thread().code()
        } else {
            HRESULT::from_win32(0)
        }
    }
}
