use windows::core::{Error, Result};
use windows::Win32::Foundation::{HANDLE, HWND};
use windows::Win32::System::Diagnostics::ToolHelp::{
    CreateToolhelp32Snapshot, Process32FirstW, Process32NextW, PROCESSENTRY32W, TH32CS_SNAPPROCESS,
};
use windows::Win32::UI::WindowsAndMessaging::{GetDesktopWindow, GetWindowThreadProcessId};

fn from_u16(s: &[u16]) -> String {
    let pos = s.iter().position(|a| a == &0u16).unwrap();
    use std::ffi::OsString;
    use std::os::windows::ffi::OsStringExt;
    let s2: OsString = OsStringExt::from_wide(&s[..pos]);
    s2.to_string_lossy().to_string()
}

struct ToolhelpSnapshot {
    first: bool,
    snapshot: HANDLE,
    pe32: PROCESSENTRY32W,
}

impl Iterator for ToolhelpSnapshot {
    type Item = PROCESSENTRY32W;
    fn next(&mut self) -> Option<Self::Item> {
        unsafe {
            if self.first {
                self.first = false;
                if Process32FirstW(self.snapshot, &mut self.pe32).is_ok() {
                    Some(self.pe32)
                } else {
                    None
                }
            } else {
                if Process32NextW(self.snapshot, &mut self.pe32).is_ok() {
                    Some(self.pe32)
                } else {
                    None
                }
            }
        }
    }
}

impl ToolhelpSnapshot {
    fn new() -> Result<ToolhelpSnapshot> {
        unsafe {
            let snapshot = CreateToolhelp32Snapshot(TH32CS_SNAPPROCESS, 0)?;
            let mut pe32 = PROCESSENTRY32W::default();
            pe32.dwSize = std::mem::size_of::<PROCESSENTRY32W>() as u32;
            Ok(ToolhelpSnapshot {
                first: true,
                snapshot,
                pe32,
            })
        }
    }
}

impl Drop for ToolhelpSnapshot {
    fn drop(&mut self) {
        unsafe {
            let _ = windows::Win32::Foundation::CloseHandle(self.snapshot);
        }
    }
}

fn get_process_name(target: HWND) -> Result<String> {
    unsafe {
        let mut pid: u32 = 0;
        GetWindowThreadProcessId(target, Some(&mut pid));

        let ths = ToolhelpSnapshot::new()?;
        for pe32 in ths {
            if pe32.th32ProcessID == pid {
                return Ok(from_u16(&pe32.szExeFile));
            }
        }

        Err(Error::from_hresult(
            windows::Win32::Foundation::ERROR_NOT_FOUND.to_hresult(),
        ))
    }
}

fn main() -> Result<()> {
    unsafe {
        let desktop = GetDesktopWindow();
        match get_process_name(desktop) {
            Ok(name) => println!("Desktop window process: {}", name),
            Err(e) => println!("Error: {}", e),
        }
    }
    Ok(())
}
