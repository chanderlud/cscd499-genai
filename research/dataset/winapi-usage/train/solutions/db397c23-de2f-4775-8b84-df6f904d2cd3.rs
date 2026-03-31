use windows::core::{Error, Result, HRESULT, PWSTR};
use windows::Win32::Security::Authorization::ConvertSidToStringSidW;
use windows::Win32::Security::PSID;

fn call_convert_sid_to_string_sid_w() -> HRESULT {
    let sid = PSID(std::ptr::null_mut());
    let mut string_sid = PWSTR(std::ptr::null_mut());
    unsafe {
        match ConvertSidToStringSidW(sid, &mut string_sid) {
            Ok(()) => HRESULT::default(),
            Err(e) => e.code(),
        }
    }
}
