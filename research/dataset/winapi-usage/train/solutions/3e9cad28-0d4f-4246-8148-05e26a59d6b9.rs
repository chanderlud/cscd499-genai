use windows::Win32::Foundation::{GetLastError, HWND, RECT, WIN32_ERROR};
use windows::Win32::Graphics::Gdi::HDC;
use windows::Win32::UI::Controls::{BeginBufferedAnimation, BP_ANIMATIONPARAMS, BP_BUFFERFORMAT};

fn call_begin_buffered_animation() -> WIN32_ERROR {
    let hwnd = HWND::default();
    let hdctarget = HDC::default();
    let rect = RECT::default();
    let format = BP_BUFFERFORMAT::default();
    let paint_params = None;
    let anim_params = BP_ANIMATIONPARAMS::default();
    let mut hdc_from = HDC::default();
    let mut hdc_to = HDC::default();

    let result = unsafe {
        BeginBufferedAnimation(
            hwnd,
            hdctarget,
            &rect,
            format,
            paint_params,
            &anim_params,
            &mut hdc_from,
            &mut hdc_to,
        )
    };

    if result == 0 {
        unsafe { GetLastError() }
    } else {
        WIN32_ERROR(0)
    }
}
