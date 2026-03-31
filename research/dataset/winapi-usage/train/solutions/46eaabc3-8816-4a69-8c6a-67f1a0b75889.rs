use windows::core::{Error, Result, HRESULT};
use windows::Win32::Security::Authorization::ConvertStringSecurityDescriptorToSecurityDescriptorW;
use windows::Win32::Security::PSECURITY_DESCRIPTOR;

fn call_convert_string_security_descriptor_to_security_descriptor_w() -> HRESULT {
    let sddl = windows::core::w!("D:(A;;GA;;;BA)");
    let mut sd = PSECURITY_DESCRIPTOR(std::ptr::null_mut());
    let result =
        unsafe { ConvertStringSecurityDescriptorToSecurityDescriptorW(sddl, 1, &mut sd, None) };
    result.map(|_| HRESULT(0)).unwrap_or_else(|e| e.code())
}
