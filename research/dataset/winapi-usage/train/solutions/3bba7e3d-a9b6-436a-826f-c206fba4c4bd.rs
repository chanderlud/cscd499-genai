use windows::core::{Error, Result};
use windows::Win32::Foundation::HWND;
use windows::Win32::Graphics::DirectComposition::{
    DCompositionAttachMouseWheelToHwnd, IDCompositionVisual,
};

fn call_d_composition_attach_mouse_wheel_to_hwnd() -> Result<()> {
    unsafe {
        DCompositionAttachMouseWheelToHwnd(None::<&IDCompositionVisual>, HWND::default(), true)?;
        Ok(())
    }
}
