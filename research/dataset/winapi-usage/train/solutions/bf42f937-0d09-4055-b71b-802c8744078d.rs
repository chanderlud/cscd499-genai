use windows::core::{Error, Result, HRESULT, PCSTR};
use windows::Win32::System::Console::AddConsoleAliasA;

fn call_add_console_alias_a() -> HRESULT {
    match unsafe {
        AddConsoleAliasA(
            PCSTR::from_raw(b"source\0".as_ptr()),
            PCSTR::from_raw(b"target\0".as_ptr()),
            PCSTR::from_raw(b"cmd.exe\0".as_ptr()),
        )
    } {
        Ok(()) => HRESULT(0),
        Err(e) => e.code(),
    }
}
