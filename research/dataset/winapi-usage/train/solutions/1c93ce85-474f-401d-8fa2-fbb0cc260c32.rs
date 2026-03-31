use windows::Win32::Foundation::WIN32_ERROR;
use windows::Win32::Graphics::Dxgi::DXGIDeclareAdapterRemovalSupport;

fn call_dxgi_declare_adapter_removal_support() -> WIN32_ERROR {
    match unsafe { DXGIDeclareAdapterRemovalSupport() } {
        Ok(()) => WIN32_ERROR(0),
        Err(e) => WIN32_ERROR(e.code().0 as u32),
    }
}
