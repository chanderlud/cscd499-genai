use windows::Win32::System::DataExchange::CloseClipboard;

unsafe fn call_close_clipboard() -> windows::core::HRESULT {
    CloseClipboard().map_or_else(|e| e.code(), |_| windows::core::HRESULT::from_win32(0))
}
