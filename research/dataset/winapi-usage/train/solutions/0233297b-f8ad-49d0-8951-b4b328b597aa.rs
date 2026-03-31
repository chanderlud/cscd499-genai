use windows::core::{Error, Result, GUID};
use windows::Win32::Foundation::HANDLE;
use windows::Win32::Graphics::Imaging::{IWICBitmap, WICCreateBitmapFromSection};

fn call_wic_create_bitmap_from_section() -> Result<IWICBitmap> {
    // SAFETY: Calling WICCreateBitmapFromSection with concrete default parameters.
    // The API returns a Result, which correctly propagates HRESULT errors.
    unsafe { WICCreateBitmapFromSection(100, 100, &GUID::default(), HANDLE::default(), 0, 0) }
}
