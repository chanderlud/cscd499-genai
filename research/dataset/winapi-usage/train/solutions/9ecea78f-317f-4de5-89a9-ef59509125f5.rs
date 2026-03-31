use windows::core::{Error, Result};
use windows::Win32::Foundation::WIN32_ERROR;
use windows::Win32::System::Variant::DosDateTimeToVariantTime;

fn call_dos_date_time_to_variant_time() -> WIN32_ERROR {
    let mut vtime: f64 = 0.0;
    // SAFETY: pvtime is a valid mutable pointer to f64, wdosdate and wdostime are valid u16s.
    let success = unsafe { DosDateTimeToVariantTime(0, 0, &mut vtime) };
    if success == 0 {
        // SAFETY: GetLastError reads the thread-local error state set by the previous Win32 call.
        unsafe { windows::Win32::Foundation::GetLastError() }
    } else {
        WIN32_ERROR(0)
    }
}
