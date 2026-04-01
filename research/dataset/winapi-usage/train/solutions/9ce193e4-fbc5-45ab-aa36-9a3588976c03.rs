use windows::Win32::System::Console::AllocConsole;

fn call_alloc_console() -> windows::core::Result<windows::core::Result<()>> {
    let inner_result = unsafe { AllocConsole() };
    Ok(inner_result)
}
