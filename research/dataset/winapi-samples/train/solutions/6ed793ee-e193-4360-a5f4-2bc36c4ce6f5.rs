use std::io;
use windows::core::PWSTR;
use windows::Win32::System::Diagnostics::Debug::{
    FormatMessageW, FORMAT_MESSAGE_FROM_SYSTEM, FORMAT_MESSAGE_IGNORE_INSERTS,
};

pub fn format_win32_error_message(error_code: u32) -> io::Result<String> {
    // Fixed-size stack buffer of 512 u16 characters
    let mut buffer = [0u16; 512];

    // SAFETY: We're calling FormatMessageW with a valid buffer pointer and size.
    // The buffer is stack-allocated and we ensure the size is correct.
    let chars_written = unsafe {
        FormatMessageW(
            FORMAT_MESSAGE_FROM_SYSTEM | FORMAT_MESSAGE_IGNORE_INSERTS,
            None,                       // No source module, use system
            error_code,                 // Pass as u32 directly
            0,                          // Default language
            PWSTR(buffer.as_mut_ptr()), // Convert to PWSTR type
            512,                        // Buffer size in characters
            None,                       // No arguments
        )
    };

    if chars_written == 0 {
        // FormatMessageW failed, get the error from GetLastError
        return Err(windows::core::Error::from_thread().into());
    }

    // Trim trailing whitespace and newlines without heap allocation
    let mut end = chars_written as usize;
    while end > 0 {
        let c = buffer[end - 1];
        if c == b'\r' as u16 || c == b'\n' as u16 || c == b' ' as u16 {
            end -= 1;
        } else {
            break;
        }
    }

    // Convert to String (this allocates, which is allowed for the return value)
    String::from_utf16(&buffer[..end]).map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))
}
