use windows::core::w;
use windows::Win32::Foundation::WIN32_ERROR;
use windows::Win32::Security::Authorization::ConvertStringSecurityDescriptorToSecurityDescriptorW;
use windows::Win32::Security::PSECURITY_DESCRIPTOR;

fn call_convert_string_security_descriptor_to_security_descriptor_w() -> WIN32_ERROR {
    let mut sd: PSECURITY_DESCRIPTOR = PSECURITY_DESCRIPTOR(std::ptr::null_mut());
    let mut size: u32 = 0;

    unsafe {
        match ConvertStringSecurityDescriptorToSecurityDescriptorW(
            w!("D:(A;;GA;;;BA)"),
            1,
            &mut sd,
            Some(&mut size),
        ) {
            Ok(()) => WIN32_ERROR(0),
            Err(e) => WIN32_ERROR(e.code().0 as u32),
        }
    }
}
