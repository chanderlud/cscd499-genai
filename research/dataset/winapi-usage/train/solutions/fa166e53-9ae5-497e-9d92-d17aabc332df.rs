use windows::core::{Error, Result};
use windows::Win32::System::Ole::BstrFromVector;

fn call_bstr_from_vector() -> Result<windows::core::BSTR> {
    // SAFETY: BstrFromVector is an unsafe Win32 API. Passing a null pointer
    // serves as a concrete parameter value for this exercise.
    unsafe { BstrFromVector(std::ptr::null()) }
}
