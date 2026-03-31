use windows::core::HRESULT;
use windows::Win32::Foundation::GetLastError;
use windows::Win32::System::DataExchange::AddAtomA;

fn call_add_atom_a() -> HRESULT {
    let atom = unsafe { AddAtomA(windows::core::s!("test")) };
    if atom == 0 {
        unsafe { HRESULT::from_win32(GetLastError().0) }
    } else {
        HRESULT::from_win32(0)
    }
}
