use windows::core::IInspectable;
use windows::core::Result;
use windows::Win32::Graphics::Dxgi::IDXGISurface;
use windows::Win32::System::WinRT::Direct3D11::CreateDirect3D11SurfaceFromDXGISurface;

unsafe fn call_create_direct3_d11_surface_from_dxgi_surface() -> Result<IInspectable> {
    let surface: IDXGISurface = unsafe { std::mem::zeroed() };

    unsafe { CreateDirect3D11SurfaceFromDXGISurface(&surface) }
}
