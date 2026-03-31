use windows::core::w;
use windows::core::{Error, Result};
use windows::Win32::Foundation::{GetLastError, WIN32_ERROR};
use windows::Win32::System::DataExchange::AddAtomW;

fn call_add_atom_w() -> WIN32_ERROR {
    let atom = unsafe { AddAtomW(w!("TestAtom")) };
    if atom == 0 {
        unsafe { GetLastError() }
    } else {
        WIN32_ERROR(0)
    }
}
