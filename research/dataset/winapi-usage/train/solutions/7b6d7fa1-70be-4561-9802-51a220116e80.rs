#![deny(warnings)]

use windows::core::{Error, Result};
use windows::Win32::Security::Authentication::Identity::{AcceptSecurityContext, ASC_REQ_FLAGS};

#[allow(dead_code)]
fn call_accept_security_context() -> Result<windows::core::HRESULT> {
    let mut context_attr: u32 = 0;

    let hresult = unsafe {
        AcceptSecurityContext(
            None,
            None,
            None,
            ASC_REQ_FLAGS(0),
            0,
            None,
            None,
            &mut context_attr,
            None,
        )
    };

    if hresult.is_err() {
        return Err(Error::from_hresult(hresult));
    }
    Ok(hresult)
}
