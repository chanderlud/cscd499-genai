use windows::core::w;
use windows::core::{Error, Result};
use windows::Win32::System::DataExchange::AddAtomW;

fn call_add_atom_w() -> Result<u16> {
    let atom = unsafe { AddAtomW(w!("TestAtom")) };
    if atom == 0 {
        Err(Error::from_thread())
    } else {
        Ok(atom)
    }
}
