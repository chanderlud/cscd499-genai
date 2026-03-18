use windows::core::HRESULT;
use windows::Win32::Foundation::WIN32_ERROR;

pub const fn try_as_win32(hr: HRESULT) -> Option<WIN32_ERROR> {
    // HRESULT layout:
    // Bit 31: Severity (0 = success, 1 = error)
    // Bits 30-29: Reserved (R) and Customer (C) flags
    // Bits 28-16: Facility (11 bits)
    // Bits 15-0: Code (16 bits)

    // FACILITY_WIN32 = 7
    const FACILITY_WIN32: i32 = 7;

    // Extract facility from bits 16-26 (11 bits)
    let facility = (hr.0 >> 16) & 0x7FF;

    if facility == FACILITY_WIN32 {
        // Extract the Win32 error code from bits 0-15
        let code = (hr.0 & 0xFFFF) as u32;
        Some(WIN32_ERROR(code))
    } else {
        None
    }
}
