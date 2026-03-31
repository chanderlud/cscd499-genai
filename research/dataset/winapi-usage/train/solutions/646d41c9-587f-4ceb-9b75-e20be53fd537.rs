#![deny(warnings)]

use windows::core::Result;
use windows::Win32::Foundation::RECT;
use windows::Win32::Graphics::Gdi::HDC;
use windows::Win32::UI::Controls::{BeginBufferedPaint, BP_BUFFERFORMAT};

#[allow(dead_code)]
fn call_begin_buffered_paint() -> Result<isize> {
    let mut hdc_out = HDC::default();
    let rect = RECT {
        left: 0,
        top: 0,
        right: 100,
        bottom: 100,
    };
    let format = BP_BUFFERFORMAT(0);

    // SAFETY: We pass valid pointers and default handles. The API expects an output HDC pointer.
    let hr = unsafe { BeginBufferedPaint(HDC::default(), &rect, format, None, &mut hdc_out) };

    windows::core::HRESULT(hr as i32).ok()?;
    Ok(hr)
}
