use windows::core::{Error, Result};
use windows::Win32::Graphics::Gdi::HENHMETAFILE;
use windows::Win32::Graphics::OpenGL::GetEnhMetaFilePixelFormat;

fn call_get_enh_meta_file_pixel_format() -> Result<u32> {
    // SAFETY: Passing a null handle and zero buffer size is safe for querying the required size.
    let result = unsafe { GetEnhMetaFilePixelFormat(HENHMETAFILE(std::ptr::null_mut()), 0, None) };
    if result == 0 {
        Err(Error::from_thread())
    } else {
        Ok(result)
    }
}
