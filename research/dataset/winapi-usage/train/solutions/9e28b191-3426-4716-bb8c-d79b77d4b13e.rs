use windows::Win32::Foundation::S_OK;
use windows::Win32::Graphics::Gdi::HDC;
use windows::Win32::Graphics::OpenGL::{
    SetPixelFormat, PFD_FLAGS, PFD_PIXEL_TYPE, PIXELFORMATDESCRIPTOR,
};

unsafe fn call_set_pixel_format() -> windows::core::HRESULT {
    let hdc = HDC(std::ptr::null_mut());

    let pfd = PIXELFORMATDESCRIPTOR {
        nSize: std::mem::size_of::<PIXELFORMATDESCRIPTOR>() as u16,
        nVersion: 1,
        dwFlags: PFD_FLAGS(0),
        iPixelType: PFD_PIXEL_TYPE(0),
        cColorBits: 0,
        cRedBits: 0,
        cRedShift: 0,
        cGreenBits: 0,
        cGreenShift: 0,
        cBlueBits: 0,
        cBlueShift: 0,
        cAlphaBits: 0,
        cAlphaShift: 0,
        cAccumBits: 0,
        cAccumRedBits: 0,
        cAccumGreenBits: 0,
        cAccumBlueBits: 0,
        cAccumAlphaBits: 0,
        cDepthBits: 0,
        cStencilBits: 0,
        cAuxBuffers: 0,
        iLayerType: 0,
        bReserved: 0,
        dwLayerMask: 0,
        dwVisibleMask: 0,
        dwDamageMask: 0,
    };

    unsafe {
        SetPixelFormat(hdc, 1, &pfd)
            .map(|_| S_OK)
            .unwrap_or_else(|e| e.code())
    }
}
