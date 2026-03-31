use windows::core::{w, HRESULT};
use windows::Win32::System::Registry::GetRegistryValueWithFallbackW;

fn call_get_registry_value_with_fallback_w() -> HRESULT {
    let err = unsafe {
        GetRegistryValueWithFallbackW(None, w!(""), None, w!(""), w!(""), 0, None, None, 0, None)
    };
    err.to_hresult()
}
