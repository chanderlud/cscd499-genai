use windows::core::{Error, Result};
use windows::Win32::Graphics::Gdi::HDC;
use windows::Win32::Graphics::OpenGL::{
    ChoosePixelFormat, PFD_DOUBLEBUFFER, PFD_DRAW_TO_WINDOW, PFD_MAIN_PLANE, PFD_SUPPORT_OPENGL,
    PFD_TYPE_RGBA, PIXELFORMATDESCRIPTOR,
};

fn call_choose_pixel_format() -> Result<i32> {
    let hdc = HDC::default();
    let pfd = PIXELFORMATDESCRIPTOR {
        nSize: std::mem::size_of::<PIXELFORMATDESCRIPTOR>() as u16,
        nVersion: 1,
        dwFlags: PFD_SUPPORT_OPENGL | PFD_DRAW_TO_WINDOW | PFD_DOUBLEBUFFER,
        iPixelType: PFD_TYPE_RGBA,
        cColorBits: 32,
        cDepthBits: 24,
        cStencilBits: 8,
        iLayerType: PFD_MAIN_PLANE.0 as u8,
        ..Default::default()
    };

    // SAFETY: ChoosePixelFormat requires a valid HDC and a pointer to a properly
    // initialized PIXELFORMATDESCRIPTOR. We provide both, and the pointer is valid
    // for the duration of the call.
    let result = unsafe { ChoosePixelFormat(hdc, &pfd) };
    if result == 0 {
        return Err(Error::from_thread());
    }
    Ok(result)
}
