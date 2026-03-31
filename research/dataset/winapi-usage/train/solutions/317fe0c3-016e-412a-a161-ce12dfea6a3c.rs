use windows::core::{Error, Result, HRESULT};
use windows::Win32::Media::{timeGetSystemTime, MMTIME};

fn call_time_get_system_time() -> HRESULT {
    let mut mmt = MMTIME::default();
    let cbmmt = std::mem::size_of::<MMTIME>() as u32;
    // SAFETY: mmt is a valid mutable pointer to an initialized MMTIME struct,
    // and cbmmt correctly specifies its size in bytes.
    let code = unsafe { timeGetSystemTime(&mut mmt, cbmmt) };
    HRESULT::from_win32(code)
}
