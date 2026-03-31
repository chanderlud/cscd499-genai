use windows::core::HRESULT;
#[allow(unused_imports)]
use windows::core::{Error, Result};
use windows::Win32::Graphics::DirectComposition::DCompositionBoostCompositorClock;

fn call_d_composition_boost_compositor_clock() -> windows::core::HRESULT {
    unsafe {
        match DCompositionBoostCompositorClock(true) {
            Ok(()) => HRESULT::default(),
            Err(e) => e.code(),
        }
    }
}
