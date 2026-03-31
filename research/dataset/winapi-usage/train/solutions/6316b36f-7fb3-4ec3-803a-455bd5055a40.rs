use windows::Win32::Foundation::WIN32_ERROR;
use windows::Win32::Security::Authorization::ConvertSidToStringSidW;
use windows::Win32::Security::PSID;

fn call_convert_sid_to_string_sid_w() -> WIN32_ERROR {
    let sid = PSID(std::ptr::null_mut());
    let mut stringsid = std::ptr::null_mut();
    // SAFETY: Passing null pointers is safe for this API call; it will simply fail and return an error.
    match unsafe { ConvertSidToStringSidW(sid, stringsid) } {
        Ok(()) => WIN32_ERROR(0),
        Err(e) => WIN32_ERROR::from_error(&e).unwrap_or_default(),
    }
}
