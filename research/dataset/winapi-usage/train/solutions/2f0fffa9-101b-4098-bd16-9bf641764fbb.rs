use windows::Win32::Foundation::{HWND, WIN32_ERROR};
use windows::Win32::Graphics::DirectComposition::{
    DCompositionAttachMouseWheelToHwnd, IDCompositionVisual,
};

fn call_d_composition_attach_mouse_wheel_to_hwnd() -> WIN32_ERROR {
    unsafe {
        match DCompositionAttachMouseWheelToHwnd(
            None::<&IDCompositionVisual>,
            HWND(std::ptr::null_mut()),
            true,
        ) {
            Ok(()) => WIN32_ERROR(0),
            Err(e) => WIN32_ERROR::from_error(&e).unwrap_or(WIN32_ERROR(0)),
        }
    }
}
