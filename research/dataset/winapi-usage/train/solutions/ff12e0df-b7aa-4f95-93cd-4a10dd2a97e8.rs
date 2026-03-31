use windows::core::HRESULT;
use windows::core::{Error, Result};
use windows::Win32::System::Variant::{ClearVariantArray, VARIANT};

fn call_clear_variant_array() -> HRESULT {
    let mut vars = [VARIANT::default(); 1];
    unsafe {
        ClearVariantArray(&mut vars);
    }
    HRESULT::default()
}
