use windows::core::{Error, Result};
use windows::Win32::System::Variant::{ClearVariantArray, VARIANT};

fn call_clear_variant_array() -> Result<()> {
    let mut vars: [VARIANT; 1] = std::array::from_fn(|_| VARIANT::default());
    // SAFETY: ClearVariantArray expects a mutable slice of VARIANTs.
    // We provide a valid slice and the function safely clears each element.
    unsafe {
        ClearVariantArray(&mut vars);
    }
    Ok(())
}
