use windows::core::{Error, Result, PWSTR};
use windows::Win32::System::Diagnostics::Debug::{FormatMessageW, FORMAT_MESSAGE_OPTIONS};

fn call_format_message_w() -> Result<u32> {
    let mut buffer = [0u16; 256];
    // SAFETY: FormatMessageW requires a valid mutable buffer pointer and size.
    // The buffer is stack-allocated and remains valid for the duration of the call.
    let result = unsafe {
        FormatMessageW(
            FORMAT_MESSAGE_OPTIONS(0x1200),
            None,
            2,
            0,
            PWSTR(buffer.as_mut_ptr()),
            buffer.len() as u32,
            None,
        )
    };
    if result == 0 {
        Err(Error::from_thread())
    } else {
        Ok(result)
    }
}
