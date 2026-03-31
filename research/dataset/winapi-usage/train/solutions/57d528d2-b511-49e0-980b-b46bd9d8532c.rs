use windows::core::{Error, Result, HRESULT};
use windows::Win32::System::Variant::DosDateTimeToVariantTime;

fn call_dos_date_time_to_variant_time() -> HRESULT {
    let mut vtime = 0.0f64;
    // SAFETY: pvtime points to a valid f64, wdosdate and wdostime are valid DOS date/time values.
    let success = unsafe { DosDateTimeToVariantTime(0, 0, &mut vtime) };
    if success == 0 {
        Error::from_thread().code()
    } else {
        HRESULT(0)
    }
}
