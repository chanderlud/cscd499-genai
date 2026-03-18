use windows::core::{Interface, Result};
use windows::Graphics::DirectX::Direct3D11::IDirect3DDevice;
use windows::Win32::Graphics::Dxgi::{IDXGIAdapter, IDXGIDevice, DXGI_ADAPTER_DESC};

pub fn get_adapter_description(device: &IDirect3DDevice) -> Result<String> {
    // Get DXGI device from IDirect3DDevice
    let dxgi_device: IDXGIDevice = device.cast()?;

    // Get adapter from DXGI device
    let adapter: IDXGIAdapter = unsafe { dxgi_device.GetAdapter()? };

    // Get adapter description
    let desc: DXGI_ADAPTER_DESC = unsafe { adapter.GetDesc()? };

    // Convert [u16; 128] description to String
    let description = {
        // Find the null terminator or use full length
        let len = desc
            .Description
            .iter()
            .position(|&c| c == 0)
            .unwrap_or(desc.Description.len());

        // Convert UTF-16 slice to String
        String::from_utf16_lossy(&desc.Description[..len])
    };

    Ok(description)
}
