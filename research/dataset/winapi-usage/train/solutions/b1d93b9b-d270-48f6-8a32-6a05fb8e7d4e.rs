use windows::core::{Error, Result};
use windows::Win32::System::Variant::DosDateTimeToVariantTime;

fn call_dos_date_time_to_variant_time() -> Result<i32> {
    let mut variant_time: f64 = 0.0;
    // SAFETY: The Win32 API writes to the provided pointer. We pass a valid mutable reference to a f64.
    let result = unsafe { DosDateTimeToVariantTime(0, 0, &mut variant_time) };

    if result == 0 {
        Err(Error::from_thread())
    } else {
        Ok(result)
    }
}
