use windows::core::{Error, Result, GUID};
use windows::Win32::Graphics::Imaging::{IWICBitmapSource, WICConvertBitmapSource};

fn call_wic_convert_bitmap_source() -> Result<IWICBitmapSource> {
    // SAFETY: Passing valid references as required by the API signature.
    // The function returns a Result, so any failure is propagated correctly.
    unsafe { WICConvertBitmapSource(&GUID::zeroed(), None::<&IWICBitmapSource>) }
}
