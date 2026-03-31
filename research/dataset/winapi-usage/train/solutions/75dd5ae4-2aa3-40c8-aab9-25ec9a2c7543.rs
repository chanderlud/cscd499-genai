use windows::core::{Error, Result};
use windows::Win32::Foundation::WIN32_ERROR;
use windows::Win32::System::Variant::{ClearVariantArray, VARIANT};

fn call_clear_variant_array() -> WIN32_ERROR {
    let mut vars = [VARIANT::default(); 1];
    unsafe {
        ClearVariantArray(&mut vars);
    }
    WIN32_ERROR(0)
}
