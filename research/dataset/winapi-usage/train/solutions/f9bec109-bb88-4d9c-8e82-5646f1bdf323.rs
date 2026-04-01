use windows::core::HRESULT;
use windows::Win32::Security::Credentials::CredFree;

fn call_cred_free() -> windows::core::HRESULT {
    // CredFree takes a pointer to free memory allocated by Cred functions
    // Using null pointer as a concrete parameter value
    unsafe {
        CredFree(std::ptr::null());
    }
    HRESULT(0)
}
