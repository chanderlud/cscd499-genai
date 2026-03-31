use windows::core::{Error, Result, HRESULT};
use windows::Win32::System::Com::{CoInitializeEx, COINIT_APARTMENTTHREADED};

fn call_co_initialize_ex() -> Result<HRESULT> {
    let hresult = unsafe { CoInitializeEx(None, COINIT_APARTMENTTHREADED) };
    hresult.ok()?;
    Ok(hresult)
}
