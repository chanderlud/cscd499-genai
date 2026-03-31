use windows::core::{w, Error, Result};
use windows::Win32::Security::Authorization::ConvertStringSecurityDescriptorToSecurityDescriptorW;
use windows::Win32::Security::PSECURITY_DESCRIPTOR;

fn call_convert_string_security_descriptor_to_security_descriptor_w() -> Result<()> {
    let mut sd: PSECURITY_DESCRIPTOR = PSECURITY_DESCRIPTOR(std::ptr::null_mut());
    unsafe {
        ConvertStringSecurityDescriptorToSecurityDescriptorW(
            w!("D:(A;;GA;;;BA)"),
            1,
            &mut sd,
            None,
        )?;
    }
    Ok(())
}
