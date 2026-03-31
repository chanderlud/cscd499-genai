#![deny(warnings)]

use windows::core::HRESULT;
use windows::Win32::Foundation::HWND;
use windows::Win32::Graphics::DirectComposition::DCompositionAttachMouseDragToHwnd;

#[allow(dead_code)]
fn call_d_composition_attach_mouse_drag_to_hwnd() -> HRESULT {
    unsafe {
        DCompositionAttachMouseDragToHwnd(None, HWND::default(), false)
            .map(|_| HRESULT(0))
            .unwrap_or_else(|e| e.code())
    }
}
