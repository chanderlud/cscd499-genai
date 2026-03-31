use windows::core::{w, Result};
use windows::Win32::System::Console::AddConsoleAliasW;

fn call_add_console_alias_w() -> Result<()> {
    unsafe {
        AddConsoleAliasW(w!("myalias"), w!("mycommand"), w!("cmd.exe"))?;
    }
    Ok(())
}
