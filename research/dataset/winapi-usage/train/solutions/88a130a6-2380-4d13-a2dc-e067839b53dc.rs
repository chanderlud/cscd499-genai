use windows::Win32::Foundation::WIN32_ERROR;
use windows::Win32::System::WinRT::Direct3D11::CreateDirect3D11SurfaceFromDXGISurface;

unsafe fn call_create_direct3_d11_surface_from_dxgi_surface() -> WIN32_ERROR {
    // CreateDirect3D11SurfaceFromDXGISurface is an unsafe Win32 API
    match CreateDirect3D11SurfaceFromDXGISurface(None) {
        Ok(_) => WIN32_ERROR(0), // S_OK
        Err(e) => WIN32_ERROR::from_error(&e).unwrap_or(WIN32_ERROR(0)),
    }
}
