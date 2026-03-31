use windows::core::{Error, Result};
use windows::Win32::Security::Authentication::Identity::{AcceptSecurityContext, ASC_REQ_FLAGS};

fn call_accept_security_context() -> windows::core::HRESULT {
    unsafe {
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
    }
}
