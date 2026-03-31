use windows::core::PWSTR;
#[allow(unused_imports)]
use windows::core::{Error, Result};
use windows::Win32::Foundation::WIN32_ERROR;
use windows::Win32::System::Diagnostics::Debug::{FormatMessageW, FORMAT_MESSAGE_OPTIONS};

fn call_format_message_w() -> WIN32_ERROR {
    // SAFETY: FormatMessageW is called with a null buffer and zero size, which is safe.
    let ret = unsafe {
        FormatMessageW(
            FORMAT_MESSAGE_OPTIONS(0),
            None,
            0,
            0,
            PWSTR(std::ptr::null_mut()),
            0,
            None,
        )
    };

    if ret == 0 {
        let err = Error::from_thread();
        WIN32_ERROR::from_error(&err).unwrap_or_default()
    } else {
        WIN32_ERROR(ret)
    }
}
