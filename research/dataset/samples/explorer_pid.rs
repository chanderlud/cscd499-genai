use windows::Win32::Foundation::CloseHandle;
use windows::Win32::System::Diagnostics::ToolHelp::{
    CreateToolhelp32Snapshot, PROCESSENTRY32W, Process32FirstW, Process32NextW, TH32CS_SNAPPROCESS,
};
use windows::core::Result;

fn main() -> Result<()> {
    // SAFETY: We are calling Win32 APIs with valid parameters and checking return values.
    unsafe {
        let snapshot = CreateToolhelp32Snapshot(TH32CS_SNAPPROCESS, 0)?;

        let mut pe32 = PROCESSENTRY32W {
            dwSize: std::mem::size_of::<PROCESSENTRY32W>() as u32,
            ..Default::default()
        };

        if Process32FirstW(snapshot, &mut pe32).is_ok() {
            loop {
                let name = String::from_utf16_lossy(
                    &pe32.szExeFile[..pe32.szExeFile.iter().position(|&c| c == 0).unwrap_or(0)],
                );

                if name.starts_with("explorer.exe") {
                    println!("Found explorer.exe with PID: {}", pe32.th32ProcessID);
                    CloseHandle(snapshot)?;
                    return Ok(());
                }

                if Process32NextW(snapshot, &mut pe32).is_err() {
                    break;
                }
            }
        }

        CloseHandle(snapshot)?;
    }

    println!("explorer.exe not found");
    Ok(())
}
