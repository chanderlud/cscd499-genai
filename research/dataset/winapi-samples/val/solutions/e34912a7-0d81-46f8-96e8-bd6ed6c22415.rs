use windows::core::{Error, Result};
use windows::Graphics::DirectX::Direct3D11::IDirect3DDevice;
use windows::Win32::Foundation::E_INVALIDARG;
use windows::Win32::Graphics::Dxgi::IDXGIDevice1;

// Helper function to get DXGI device from WinRT Direct3D device
// This is a placeholder - the actual implementation would use the provided pattern
fn get_d3d_interface_from_object(_device: &IDirect3DDevice) -> Result<IDXGIDevice1> {
    // Implementation would use the provided pattern to extract IDXGIDevice1
    // For now, this is a stub that would be replaced with actual code
    unimplemented!("This function should be implemented using the provided pattern")
}

pub fn set_max_frame_latency(device: &IDirect3DDevice, max_latency: u32) -> Result<()> {
    if max_latency == 0 {
        return Err(Error::from_hresult(E_INVALIDARG));
    }

    let dxgi_device = get_d3d_interface_from_object(device)?;

    // SAFETY: We're calling a COM method on a valid DXGI device interface
    unsafe {
        dxgi_device.SetMaximumFrameLatency(max_latency)?;
    }

    Ok(())
}
