use std::path::Path;
use windows::core::GUID;
use windows::core::{Error, Result, PCWSTR};
use windows::Win32::Foundation::{E_INVALIDARG, PROPERTYKEY};
use windows::Win32::System::Com::StructuredStorage::PropVariantClear;
use windows::Win32::System::Com::{CoInitializeEx, CoUninitialize, COINIT_MULTITHREADED};
use windows::Win32::System::Variant::VT_UI8;
use windows::Win32::UI::Shell::PropertiesSystem::{
    IPropertyStore, SHGetPropertyStoreFromParsingName, GPS_DEFAULT,
};

fn wide_null(s: &std::ffi::OsStr) -> Vec<u16> {
    use std::{iter::once, os::windows::ffi::OsStrExt};
    s.encode_wide().chain(once(0)).collect()
}

// Define PKEY_Size manually since EnhancedStorage feature is not enabled
const PKEY_Size: PROPERTYKEY = PROPERTYKEY {
    fmtid: GUID::from_u128(0xb725f130_47ef_101a_a5f1_02608c9eebac),
    pid: 12,
};

pub fn property_size(path: &Path) -> Result<u64> {
    // Initialize COM for the current thread
    unsafe {
        CoInitializeEx(None, COINIT_MULTITHREADED).ok()?;
    }

    // Ensure COM is uninitialized when we exit
    struct ComGuard;
    impl Drop for ComGuard {
        fn drop(&mut self) {
            unsafe {
                CoUninitialize();
            }
        }
    }
    let _guard = ComGuard;

    // Convert path to wide string
    let wide_path = wide_null(path.as_os_str());
    let pcwstr = PCWSTR(wide_path.as_ptr());

    // Get property store from parsing name
    let property_store: IPropertyStore =
        unsafe { SHGetPropertyStoreFromParsingName(pcwstr, None, GPS_DEFAULT)? };

    // Get the PKEY_Size value
    let prop_variant = unsafe { property_store.GetValue(&PKEY_Size)? };

    // Extract the size value from PROPVARIANT
    // PKEY_Size should be VT_UI8 (unsigned 64-bit integer)
    let size = unsafe {
        if prop_variant.vt() != VT_UI8 {
            return Err(Error::from_hresult(E_INVALIDARG));
        }
        // uhVal is already a u64 for VT_UI8 - no .QuadPart needed
        prop_variant.Anonymous.Anonymous.Anonymous.uhVal
    };

    // Clean up the PROPVARIANT
    let mut prop_variant = prop_variant;
    unsafe {
        let _ = PropVariantClear(&mut prop_variant);
    }

    Ok(size)
}
