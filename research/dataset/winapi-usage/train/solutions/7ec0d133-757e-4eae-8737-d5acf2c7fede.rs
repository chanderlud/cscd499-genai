use windows::core::{Error, Result};
use windows::Win32::Graphics::DirectComposition::{DCompositionCreateDevice, IDCompositionDevice};
use windows::Win32::Graphics::Dxgi::IDXGIDevice;

fn call_d_composition_create_device() -> Result<IDCompositionDevice> {
    // SAFETY: Passing None is valid and instructs the API to create a device using the default adapter.
    unsafe { DCompositionCreateDevice(None::<&IDXGIDevice>) }
}
