use windows::core::{Error, Result};
use windows::Win32::Foundation::WIN32_ERROR;
use windows::Win32::Security::{FreeSid, PSID};

fn call_free_sid() -> WIN32_ERROR {
    unsafe {
        FreeSid(PSID(std::ptr::null_mut()));
    }
    WIN32_ERROR(0)
}
