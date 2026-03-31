use windows::core::Error;
use windows::Win32::Foundation::WIN32_ERROR;
use windows::Win32::System::DataExchange::AddAtomA;

fn call_add_atom_a() -> WIN32_ERROR {
    let atom = unsafe { AddAtomA(windows::core::s!("test")) };
    if atom == 0 {
        let err = Error::from_thread();
        return WIN32_ERROR(err.code().0 as u32);
    }
    WIN32_ERROR(0)
}
