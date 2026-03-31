use windows::core::HRESULT;
use windows::Win32::System::Console::AddConsoleAliasW;

fn call_add_console_alias_w() -> HRESULT {
    let source = windows::core::w!("source");
    let target = windows::core::w!("target");
    let exename = windows::core::w!("cmd.exe");

    // SAFETY: The `w!` macro guarantees null-terminated wide string literals,
    // which satisfy the `PCWSTR` parameter requirements of `AddConsoleAliasW`.
    unsafe {
        match AddConsoleAliasW(source, target, exename) {
            Ok(()) => HRESULT::default(),
            Err(e) => e.code(),
        }
    }
}
