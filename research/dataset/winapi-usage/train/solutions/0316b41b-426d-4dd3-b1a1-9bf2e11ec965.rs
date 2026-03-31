use windows::Win32::Foundation::{GetLastError, RECT, WIN32_ERROR};
use windows::Win32::Graphics::Gdi::HDC;
use windows::Win32::UI::Controls::{BeginBufferedPaint, BP_BUFFERFORMAT};

fn call_begin_buffered_paint() -> WIN32_ERROR {
    let hdc_target = HDC::default();
    let rect = RECT::default();
    let format = BP_BUFFERFORMAT(0);
    let mut hdc_buffer = HDC::default();

    let hdc = unsafe { BeginBufferedPaint(hdc_target, &rect, format, None, &mut hdc_buffer) };

    if hdc == 0 {
        unsafe { GetLastError() }
    } else {
        WIN32_ERROR::default()
    }
}
