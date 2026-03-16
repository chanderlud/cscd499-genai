use windows::core::{Result, Error};
use windows::Win32::Foundation::{HLOCAL, LocalFree};
use windows::Win32::System::Diagnostics::Debug::{
    FormatMessageW, FORMAT_MESSAGE_ALLOCATE_BUFFER, FORMAT_MESSAGE_FROM_SYSTEM,
    FORMAT_MESSAGE_IGNORE_INSERTS,
};

pub fn format_win32_error_message(error_code: u32) -> std::io::Result<String> {
    let mut buffer: *mut u16 = std::ptr::null_mut();
    
    // SAFETY: FormatMessageW with FORMAT_MESSAGE_ALLOCATE_BUFFER allocates memory
    // that we must free with LocalFree. The function is thread-safe for our usage.
    let chars_written = unsafe {
        FormatMessageW(
            FORMAT_MESSAGE_ALLOCATE_BUFFER | FORMAT_MESSAGE_FROM_SYSTEM | FORMAT_MESSAGE_IGNORE_INSERTS,
            std::ptr::null(),
            error_code,
            0,
            &mut buffer as *mut *mut u16 as *mut u16,
            0,
            std::ptr::null(),
        )
    };
    
    if chars_written == 0 {
        // FormatMessageW failed, get the error
        let err = Error::from_win32();
        return Err(std::io::Error::from_raw_os_error(err.code().0));
    }
    
    // SAFETY: buffer is valid for chars_written characters (excluding null terminator)
    // and was allocated by FormatMessageW
    let message_slice = unsafe { std::slice::from_raw_parts(buffer, chars_written as usize) };
    let message = String::from_utf16_lossy(message_slice);
    let trimmed_message = message.trim_end().to_string();
    
    // SAFETY: buffer was allocated by FormatMessageW with FORMAT_MESSAGE_ALLOCATE_BUFFER
    // and must be freed with LocalFree
    unsafe {
        let _ = LocalFree(HLOCAL(buffer as _));
    }
    
    Ok(trimmed_message)
}