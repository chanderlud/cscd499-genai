use windows::core::{w, Error, Result};
use windows::Win32::Foundation::WIN32_ERROR;
use windows::Win32::System::Registry::GetRegistryValueWithFallbackW;

fn call_get_registry_value_with_fallback_w() -> Result<WIN32_ERROR> {
    // SAFETY: Calling Win32 API with valid null-terminated wide strings and null pointers for optional outputs.
    let win32_err = unsafe {
        GetRegistryValueWithFallbackW(
            None,
            w!("Software\\Primary"),
            None,
            w!("Software\\Fallback"),
            w!("TestValue"),
            0,
            None,
            None,
            0,
            None,
        )
    };
    if win32_err.0 != 0 {
        return Err(Error::from_hresult(win32_err.to_hresult()));
    }
    Ok(win32_err)
}
