use windows::core::{Error, HRESULT};
use windows::Win32::Foundation::WIN32_ERROR;

pub fn try_as_win32(err: &Error) -> Option<WIN32_ERROR> {
    let hr = err.code();

    // Check if this is a FACILITY_WIN32 HRESULT
    // FACILITY_WIN32 = 7, which occupies bits 16-25 of the HRESULT
    // The Win32 error code is in bits 0-15
    let facility = (hr.0 >> 16) & 0x7FF; // Extract bits 16-26 (11 bits total)

    if facility == 7 {
        // Extract the Win32 error code from bits 0-15
        let code = (hr.0 & 0xFFFF) as u32;
        Some(WIN32_ERROR(code))
    } else {
        None
    }
}
