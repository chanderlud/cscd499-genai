#![deny(warnings)]

use windows::Win32::Foundation::WIN32_ERROR;
use windows::Win32::System::Com::{CoInitializeEx, COINIT_APARTMENTTHREADED};

#[allow(dead_code)]
fn call_co_initialize_ex() -> WIN32_ERROR {
    let hresult = unsafe { CoInitializeEx(None, COINIT_APARTMENTTHREADED) };
    WIN32_ERROR(hresult.0 as u32)
}
