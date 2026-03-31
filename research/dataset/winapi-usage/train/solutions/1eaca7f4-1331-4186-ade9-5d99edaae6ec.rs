use windows::core::Result;
use windows::Win32::Foundation::WIN32_ERROR;
use windows::Win32::Graphics::DirectComposition::{DCompositionCreateDevice, IDCompositionDevice};
use windows::Win32::Graphics::Dxgi::IDXGIDevice;

fn call_d_composition_create_device() -> WIN32_ERROR {
    let result: Result<IDCompositionDevice> =
        unsafe { DCompositionCreateDevice(None::<&IDXGIDevice>) };
    match result {
        Ok(_) => WIN32_ERROR(0),
        Err(e) => WIN32_ERROR(e.code().0 as u32),
    }
}
