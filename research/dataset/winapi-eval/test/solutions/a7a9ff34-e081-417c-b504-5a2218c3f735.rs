use windows::Win32::Foundation::{SetLastError, WIN32_ERROR};

pub fn set_last_error_code(code: u32) {
    unsafe {
        SetLastError(WIN32_ERROR(code));
    }
}
