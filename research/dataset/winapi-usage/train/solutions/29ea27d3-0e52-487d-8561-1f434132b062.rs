use windows::Win32::Foundation::WIN32_ERROR;
use windows::Win32::Graphics::Imaging::{IWICMetadataWriter, WICGetMetadataContentSize};

fn call_wic_get_metadata_content_size() -> WIN32_ERROR {
    let result =
        unsafe { WICGetMetadataContentSize(std::ptr::null(), None::<&IWICMetadataWriter>) };
    match result {
        Ok(_) => WIN32_ERROR(0),
        Err(e) => WIN32_ERROR::from_error(&e).unwrap_or_default(),
    }
}
