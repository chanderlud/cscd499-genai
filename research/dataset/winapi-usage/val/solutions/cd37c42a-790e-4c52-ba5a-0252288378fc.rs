use windows::Win32::Foundation::{HWND, WIN32_ERROR};
use windows::Win32::Graphics::DirectComposition::{
    DCompositionAttachMouseDragToHwnd, IDCompositionVisual,
};

fn call_d_composition_attach_mouse_drag_to_hwnd() -> WIN32_ERROR {
    let visual = None::<&IDCompositionVisual>;
    let hwnd = HWND(std::ptr::null_mut());
    let enable = false;

    match unsafe { DCompositionAttachMouseDragToHwnd(visual, hwnd, enable) } {
        Ok(()) => WIN32_ERROR(0),
        Err(e) => WIN32_ERROR::from_error(&e).unwrap_or_default(),
    }
}
