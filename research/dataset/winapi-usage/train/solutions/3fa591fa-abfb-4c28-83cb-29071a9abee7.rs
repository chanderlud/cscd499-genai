use windows::Win32::System::DataExchange::CloseClipboard;

fn call_close_clipboard() -> windows::core::Result<()> {
    let result = unsafe { CloseClipboard() };
    result
}
