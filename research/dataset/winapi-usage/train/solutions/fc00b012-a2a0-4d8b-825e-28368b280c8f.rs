use windows::core::HRESULT;
use windows::core::{Error, Result};
use windows::Win32::Foundation::S_OK;
use windows::Win32::Security::{FreeSid, PSID};

fn call_free_sid() -> HRESULT {
    unsafe {
        let _ = FreeSid(PSID(std::ptr::null_mut()));
    }
    S_OK
}
