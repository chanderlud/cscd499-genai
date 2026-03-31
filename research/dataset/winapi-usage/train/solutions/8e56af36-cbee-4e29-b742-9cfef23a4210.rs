use windows::core::HRESULT;
use windows::Win32::Foundation::HWND;
use windows::Win32::Graphics::DirectComposition::{
    DCompositionAttachMouseWheelToHwnd, IDCompositionVisual,
};

fn call_d_composition_attach_mouse_wheel_to_hwnd() -> HRESULT {
    // SAFETY: Calling the API with null/default parameters. The function will either succeed
    // or return an error, which we safely convert to an HRESULT.
    unsafe {
        match DCompositionAttachMouseWheelToHwnd(
            None::<&IDCompositionVisual>,
            HWND::default(),
            true,
        ) {
            Ok(()) => HRESULT(0),
            Err(e) => e.code(),
        }
    }
}
