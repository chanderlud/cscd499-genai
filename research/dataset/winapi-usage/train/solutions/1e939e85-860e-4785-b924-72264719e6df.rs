use windows::Win32::Foundation::{HWND, WIN32_ERROR};
use windows::Win32::System::DataExchange::AddClipboardFormatListener;

fn call_add_clipboard_format_listener() -> WIN32_ERROR {
    match unsafe { AddClipboardFormatListener(HWND::default()) } {
        Ok(()) => WIN32_ERROR(0),
        Err(e) => WIN32_ERROR::from_error(&e).unwrap_or_default(),
    }
}
