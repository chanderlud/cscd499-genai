use windows::core::{Error, Result};
use windows::Win32::Graphics::Dxgi::DXGIDeclareAdapterRemovalSupport;

fn call_dxgi_declare_adapter_removal_support() -> Result<Result<()>> {
    // SAFETY: DXGIDeclareAdapterRemovalSupport is safe to call as it only registers
    // support for adapter removal with the OS and requires no special preconditions.
    Ok(unsafe { DXGIDeclareAdapterRemovalSupport() })
}
