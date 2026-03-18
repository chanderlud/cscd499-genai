use windows::core::{Interface, Result};
use windows::Graphics::DirectX::Direct3D11::IDirect3DSurface;
use windows::Win32::Graphics::Direct3D11::*;
use windows::Win32::Graphics::Dxgi::Common::{DXGI_FORMAT, DXGI_SAMPLE_DESC};
use windows::Win32::Graphics::Dxgi::IDXGISurface;
use windows::Win32::System::WinRT::Direct3D11::CreateDirect3D11SurfaceFromDXGISurface;

// Helper function to get D3D11 device from WinRT device (assumed to be provided)
fn get_d3d_interface_from_object(
    _device: &windows::Graphics::DirectX::Direct3D11::IDirect3DDevice,
) -> Result<ID3D11Device> {
    // Implementation would be provided elsewhere
    unimplemented!()
}

pub fn create_texture_surface(
    device: &windows::Graphics::DirectX::Direct3D11::IDirect3DDevice,
    width: u32,
    height: u32,
    format: DXGI_FORMAT,
) -> Result<IDirect3DSurface> {
    // Get the underlying D3D11 device
    let d3d_device = get_d3d_interface_from_object(device)?;

    // Create texture description
    let texture_desc = D3D11_TEXTURE2D_DESC {
        Width: width,
        Height: height,
        MipLevels: 1,
        ArraySize: 1,
        Format: format,
        SampleDesc: DXGI_SAMPLE_DESC {
            Count: 1,
            Quality: 0,
        },
        Usage: D3D11_USAGE_DEFAULT,
        BindFlags: D3D11_BIND_SHADER_RESOURCE.0 as u32,
        CPUAccessFlags: 0,
        MiscFlags: 0,
    };

    // Create the texture
    let texture: ID3D11Texture2D = unsafe {
        let mut texture = None;
        d3d_device.CreateTexture2D(&texture_desc, None, Some(&mut texture))?;
        texture.unwrap()
    };

    // Convert to DXGI surface
    let dxgi_surface: IDXGISurface = texture.cast()?;

    // Convert to WinRT Direct3D surface
    let direct3d_surface = unsafe {
        let inspectable = CreateDirect3D11SurfaceFromDXGISurface(&dxgi_surface)?;
        inspectable.cast()?
    };

    Ok(direct3d_surface)
}
