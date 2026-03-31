use windows::core::{Error, Result, HRESULT, PWSTR};
use windows::Win32::System::Diagnostics::Debug::{FormatMessageW, FORMAT_MESSAGE_OPTIONS};

fn call_format_message_w() -> HRESULT {
    unsafe {
        let len = FormatMessageW(
            FORMAT_MESSAGE_OPTIONS(0x1000),
            None,
            0,
            0,
            PWSTR(std::ptr::null_mut()),
            0,
            None,
        );
        if len == 0 {
            Error::from_thread().code()
        } else {
            HRESULT::from_win32(0)
        }
    }
}
