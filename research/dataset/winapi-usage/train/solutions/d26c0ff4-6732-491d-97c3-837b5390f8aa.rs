use windows::core::{Error, Result, HRESULT};
use windows::Win32::System::Ole::BstrFromVector;

fn call_bstr_from_vector() -> windows::core::HRESULT {
    match unsafe { BstrFromVector(std::ptr::null()) } {
        Ok(_) => HRESULT(0),
        Err(e) => e.code(),
    }
}
