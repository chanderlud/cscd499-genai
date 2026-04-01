use windows::Win32::Foundation::{ERROR_SUCCESS, WIN32_ERROR};
use windows::Win32::System::DataExchange::CloseClipboard;

unsafe fn call_close_clipboard() -> WIN32_ERROR {
    match CloseClipboard() {
        Ok(()) => ERROR_SUCCESS,
        Err(e) => WIN32_ERROR(e.code().0 as u32),
    }
}
