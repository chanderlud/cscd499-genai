#![deny(warnings)]

use windows::core::Result;
use windows::Win32::System::Com::CLSIDFromProgID;

#[allow(dead_code)]
fn call_clsid_from_prog_id() -> Result<windows::core::GUID> {
    // SAFETY: CLSIDFromProgID requires a valid null-terminated wide string pointer.
    // The `w!` macro provides a compile-time wide string literal that satisfies this requirement.
    let guid = unsafe { CLSIDFromProgID(windows::core::w!("WScript.Shell"))? };
    Ok(guid)
}
