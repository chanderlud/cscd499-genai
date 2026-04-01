use windows::Win32::Graphics::Dxgi::IDXGISurface;
use windows::Win32::System::WinRT::Direct3D11::CreateDirect3D11SurfaceFromDXGISurface;

fn call_create_direct3_d11_surface_from_dxgi_surface() -> windows::core::HRESULT {
    unsafe { CreateDirect3D11SurfaceFromDXGISurface(Option::<&IDXGISurface>::None).into() }
}
