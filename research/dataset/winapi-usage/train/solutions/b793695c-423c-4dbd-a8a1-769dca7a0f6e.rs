use windows::core::{Error, Result};
use windows::Win32::Foundation::{HWND, RECT};
use windows::Win32::Graphics::Gdi::HDC;
use windows::Win32::UI::Controls::{
    BeginBufferedAnimation, BP_ANIMATIONPARAMS, BP_ANIMATIONSTYLE, BP_BUFFERFORMAT,
};

#[allow(dead_code)]
fn call_begin_buffered_animation() -> Result<isize> {
    let rect = RECT {
        left: 0,
        top: 0,
        right: 100,
        bottom: 100,
    };
    let mut anim_params = BP_ANIMATIONPARAMS {
        cbSize: std::mem::size_of::<BP_ANIMATIONPARAMS>() as u32,
        style: BP_ANIMATIONSTYLE(0),
        dwDuration: 500,
        dwFlags: 0,
    };
    let mut hdc_from = HDC::default();
    let mut hdc_to = HDC::default();

    // SAFETY: All pointers are valid, structs are properly initialized with correct sizes,
    // and parameters match the API requirements for BeginBufferedAnimation.
    let result = unsafe {
        BeginBufferedAnimation(
            HWND::default(),
            HDC::default(),
            &rect,
            BP_BUFFERFORMAT(1),
            None,
            &mut anim_params,
            &mut hdc_from,
            &mut hdc_to,
        )
    };

    if result == 0 {
        Err(Error::from_thread())
    } else {
        Ok(result)
    }
}
