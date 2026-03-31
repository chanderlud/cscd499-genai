#![deny(warnings)]

use windows::core::{Result, GUID};
use windows::Win32::Graphics::Imaging::{IWICMetadataWriter, WICGetMetadataContentSize};

#[allow(dead_code)]
fn call_wic_get_metadata_content_size() -> Result<u64> {
    let guid = GUID::zeroed();
    // SAFETY: WICGetMetadataContentSize expects a valid pointer for guidcontainerformat.
    // Passing a pointer to a zeroed GUID and None for the writer is safe.
    unsafe { WICGetMetadataContentSize(&guid, None::<&IWICMetadataWriter>) }
}
