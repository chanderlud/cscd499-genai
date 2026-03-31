use windows::core::{Error, Result};
use windows::Win32::System::Registry::GetRegistryValueWithFallbackW;

fn call_get_registry_value_with_fallback_w() -> windows::Win32::Foundation::WIN32_ERROR {
    unsafe {
        GetRegistryValueWithFallbackW(
            None,
            windows::core::w!("Software\\Primary"),
            None,
            windows::core::w!("Software\\Fallback"),
            windows::core::w!("ValueName"),
            0,
            None,
            None,
            0,
            None,
        )
    }
}
