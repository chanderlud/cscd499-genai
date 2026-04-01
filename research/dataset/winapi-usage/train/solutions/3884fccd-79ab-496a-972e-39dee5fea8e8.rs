use windows::Win32::System::Console::AllocConsole;

unsafe fn call_alloc_console() -> windows::core::HRESULT {
    match AllocConsole() {
        Ok(()) => windows::core::HRESULT::from_win32(0),
        Err(e) => e.code(),
    }
}
