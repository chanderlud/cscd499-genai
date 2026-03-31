use windows::core::{w, Error, Result, HRESULT};
use windows::Win32::Foundation::S_OK;
use windows::Win32::System::DataExchange::AddAtomW;

fn call_add_atom_w() -> HRESULT {
    let atom = unsafe { AddAtomW(w!("TestAtom")) };
    if atom == 0 {
        Error::from_thread().code()
    } else {
        S_OK
    }
}
