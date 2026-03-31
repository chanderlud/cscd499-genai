#![deny(warnings)]

use windows::core::Result;
use windows::Win32::Foundation::HWND;
use windows::Win32::System::DataExchange::AddClipboardFormatListener;

#[allow(dead_code)]
fn call_add_clipboard_format_listener() -> Result<Result<()>> {
    let hwnd = HWND::default();
    // SAFETY: The API expects a window handle. Passing a default/null HWND is safe because
    // the function validates the handle and returns a Result error if it is invalid,
    // thus avoiding undefined behavior.
    Ok(unsafe { AddClipboardFormatListener(hwnd) })
}
