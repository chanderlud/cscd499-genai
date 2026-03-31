use windows::core::{Error, Result, PWSTR};
use windows::Win32::Security::Authorization::ConvertSidToStringSidW;
use windows::Win32::Security::PSID;

#[allow(dead_code)]
fn call_convert_sid_to_string_sid_w() -> Result<()> {
    let mut out_str = PWSTR(std::ptr::null_mut());
    unsafe {
        ConvertSidToStringSidW(PSID(std::ptr::null_mut()), &mut out_str)?;
    }
    Ok(())
}
