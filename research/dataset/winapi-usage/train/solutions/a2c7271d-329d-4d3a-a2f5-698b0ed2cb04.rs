use windows::core::w;
use windows::Win32::Foundation::WIN32_ERROR;
use windows::Win32::System::Console::AddConsoleAliasW;

fn call_add_console_alias_w() -> WIN32_ERROR {
    // SAFETY: AddConsoleAliasW is safe to call with valid null-terminated wide strings.
    match unsafe { AddConsoleAliasW(w!("source"), w!("target"), w!("cmd.exe")) } {
        Ok(()) => WIN32_ERROR(0),
        Err(e) => WIN32_ERROR::from_error(&e).unwrap_or_else(|| WIN32_ERROR(e.code().0 as u32)),
    }
}
