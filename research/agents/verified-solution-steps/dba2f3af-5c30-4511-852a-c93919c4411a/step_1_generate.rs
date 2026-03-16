use std::path::Path;
use windows::core::{Result, Error, HRESULT, PCWSTR};
use windows::Win32::System::Com::{CoInitializeEx, CoUninitialize, COINIT_MULTITHREADED, IPropertyStore, PROPVARIANT, VT_UI8};
use windows::Win32::UI::Shell::PropertiesSystem::{PSGetPropertyStoreFromParsingName, PKEY_Size, GPS_DEFAULT};

fn wide_null(s: &std::ffi::OsStr) -> Vec<u16> {
    use std::{ffi::OsStr, iter::once, os::windows::ffi::OsStrExt};
    s.encode_wide().chain(once(0)).collect()
}

pub fn property_size(path: &Path) -> Result<u64> {
    // Initialize COM for the current thread
    unsafe {
        CoInitializeEx(None, COINIT_MULTITHREADED).ok()?;
    }
    
    // Ensure COM is uninitialized when we exit
    struct ComGuard;
    impl Drop for ComGuard {
        fn drop(&mut self) {
            unsafe { CoUninitialize(); }
        }
    }
    let _guard = ComGuard;
    
    // Convert path to wide string
    let wide_path = wide_null(path.as_os_str());
    let pcwstr = PCWSTR(wide_path.as_ptr());
    
    // Get property store from parsing name
    let property_store: IPropertyStore = unsafe {
        PSGetPropertyStoreFromParsingName(pcwstr, None, GPS_DEFAULT)?
    };
    
    // Get the PKEY_Size value
    let mut prop_variant = PROPVARIANT::default();
    unsafe {
        property_store.GetValue(&PKEY_Size, &mut prop_variant)?;
    }
    
    // Extract the size value from PROPVARIANT
    // PKEY_Size should be VT_UI8 (unsigned 64-bit integer)
    let size = unsafe {
        if prop_variant.vt != VT_UI8 {
            return Err(Error::from_hresult(HRESULT::from_win32(0x80070057))); // E_INVALIDARG
        }
        *prop_variant.Anonymous.Anonymous.uhVal.QuadPart() as u64
    };
    
    Ok(size)
}