use windows::core::HRESULT;
use windows::Win32::Graphics::Imaging::{IWICMetadataWriter, WICGetMetadataContentSize};

fn call_wic_get_metadata_content_size() -> HRESULT {
    unsafe {
        WICGetMetadataContentSize(
            std::ptr::null::<windows::core::GUID>(),
            None::<&IWICMetadataWriter>,
        )
    }
    .map(|_| HRESULT(0))
    .unwrap_or_else(|e| e.code())
}
