use windows::core::Result;
use windows::Win32::System::Console::AddConsoleAliasA;

fn call_add_console_alias_a() -> Result<()> {
    // SAFETY: AddConsoleAliasA expects null-terminated ANSI strings.
    // We provide static byte slices with explicit null terminators,
    // which safely outlive the function call.
    unsafe {
        AddConsoleAliasA(
            windows::core::PCSTR::from_raw(b"my_alias\0".as_ptr()),
            windows::core::PCSTR::from_raw(b"my_target\0".as_ptr()),
            windows::core::PCSTR::from_raw(b"cmd.exe\0".as_ptr()),
        )
    }
}
