use windows::core::Error;
use windows::Win32::Foundation::WIN32_ERROR;
use windows::Win32::Security::Authentication::Identity::{AcceptSecurityContext, ASC_REQ_FLAGS};

fn call_accept_security_context() -> WIN32_ERROR {
    let hresult = unsafe {
        AcceptSecurityContext(
            None,
            None,
            None,
            ASC_REQ_FLAGS(0),
            0,
            None,
            None,
            std::ptr::null_mut(),
            None,
        )
    };

    if hresult.is_ok() {
        WIN32_ERROR(0)
    } else {
        WIN32_ERROR::from_error(&Error::from_hresult(hresult)).unwrap_or(WIN32_ERROR(0))
    }
}
