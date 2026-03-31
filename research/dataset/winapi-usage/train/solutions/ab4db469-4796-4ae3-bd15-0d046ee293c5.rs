use windows::core::{Error, Result, HRESULT};
use windows::Win32::Foundation::HWND;
use windows::Win32::System::DataExchange::AddClipboardFormatListener;

fn call_add_clipboard_format_listener() -> HRESULT {
    let hwnd = HWND::default();
    match unsafe { AddClipboardFormatListener(hwnd) } {
        Ok(()) => HRESULT(0),
        Err(e) => e.code(),
    }
}
