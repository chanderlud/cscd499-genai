#![allow(dead_code)]

use windows::core::PCSTR;
use windows::Win32::Foundation::WIN32_ERROR;
use windows::Win32::System::Console::AddConsoleAliasA;

fn call_add_console_alias_a() -> WIN32_ERROR {
    let source = PCSTR::from_raw(b"source\0".as_ptr());
    let target = PCSTR::from_raw(b"target\0".as_ptr());
    let exename = PCSTR::from_raw(b"cmd.exe\0".as_ptr());

    match unsafe { AddConsoleAliasA(source, target, exename) } {
        Ok(()) => WIN32_ERROR(0),
        Err(e) => WIN32_ERROR::from_error(&e).unwrap_or_default(),
    }
}
