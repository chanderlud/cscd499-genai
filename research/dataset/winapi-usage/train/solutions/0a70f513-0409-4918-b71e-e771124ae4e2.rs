use windows::core::{Error, Result, PCSTR};
use windows::Win32::System::DataExchange::AddAtomA;

fn call_add_atom_a() -> Result<u16> {
    // SAFETY: The C-string literal is null-terminated and valid for the duration of the call.
    let atom = unsafe { AddAtomA(PCSTR(c"MyAtom".as_ptr() as *const u8)) };
    if atom == 0 {
        Err(Error::from_thread())
    } else {
        Ok(atom)
    }
}
