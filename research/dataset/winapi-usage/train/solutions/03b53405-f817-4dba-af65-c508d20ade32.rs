#![allow(dead_code)]

use windows::core::{Error, HRESULT};
use windows::Win32::Foundation::{RECT, S_OK};
use windows::Win32::Graphics::Gdi::HDC;
use windows::Win32::UI::Controls::{BeginBufferedPaint, BP_BUFFERFORMAT};

fn call_begin_buffered_paint() -> HRESULT {
    let mut hdc = HDC::default();
    let rect = RECT {
        left: 0,
        top: 0,
        right: 100,
        bottom: 100,
    };

    let result =
        unsafe { BeginBufferedPaint(HDC::default(), &rect, BP_BUFFERFORMAT(0), None, &mut hdc) };

    if result == 0 {
        S_OK
    } else {
        Error::from_thread().code()
    }
}
