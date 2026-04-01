use windows::Win32::Foundation::{ERROR_SUCCESS, WIN32_ERROR};
use windows::Win32::Security::Credentials::CredFree;

fn call_cred_free() -> WIN32_ERROR {
    unsafe {
        CredFree(std::ptr::null());
    }
    ERROR_SUCCESS
}
