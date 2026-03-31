use windows::core::{Error, Result};
use windows::Win32::Foundation::HWND;
use windows::Win32::Graphics::DirectComposition::DCompositionAttachMouseDragToHwnd;

fn call_d_composition_attach_mouse_drag_to_hwnd() -> Result<()> {
    unsafe {
        DCompositionAttachMouseDragToHwnd(
            None::<&windows::Win32::Graphics::DirectComposition::IDCompositionVisual>,
            HWND::default(),
            false,
        )
    }
}
