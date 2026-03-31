use windows::core::{Error, Result};
use windows::Win32::Foundation::WIN32_ERROR;
use windows::Win32::Graphics::DirectComposition::DCompositionBoostCompositorClock;

fn call_d_composition_boost_compositor_clock() -> WIN32_ERROR {
    match unsafe { DCompositionBoostCompositorClock(true) } {
        Ok(()) => WIN32_ERROR(0),
        Err(e) => WIN32_ERROR(e.code().0 as u32),
    }
}
