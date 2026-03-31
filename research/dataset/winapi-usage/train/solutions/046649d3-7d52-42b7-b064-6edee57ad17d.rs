use windows::core::Result;
use windows::Win32::Graphics::DirectComposition::DCompositionBoostCompositorClock;

fn call_d_composition_boost_compositor_clock() -> Result<Result<()>> {
    // SAFETY: DCompositionBoostCompositorClock is a standard Win32 API that safely accepts a boolean.
    Ok(unsafe { DCompositionBoostCompositorClock(true) })
}
