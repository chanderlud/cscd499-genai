use windows::core::{Error, Result, HRESULT};
use windows::Win32::Graphics::Gdi::HENHMETAFILE;
use windows::Win32::Graphics::OpenGL::GetEnhMetaFilePixelFormat;

fn call_get_enh_meta_file_pixel_format() -> HRESULT {
    let hemf = HENHMETAFILE(std::ptr::null_mut());
    let _ = unsafe { GetEnhMetaFilePixelFormat(hemf, 0, None) };
    HRESULT::from_win32(0)
}
