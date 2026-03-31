#![deny(warnings)]

use windows::core::HRESULT;
use windows::Win32::Graphics::DirectComposition::{DCompositionCreateDevice, IDCompositionDevice};
use windows::Win32::Graphics::Dxgi::IDXGIDevice;

#[allow(dead_code)]
fn call_d_composition_create_device() -> HRESULT {
    unsafe {
        match DCompositionCreateDevice::<_, IDCompositionDevice>(None::<&IDXGIDevice>) {
            Ok(_) => HRESULT(0),
            Err(e) => e.code(),
        }
    }
}
