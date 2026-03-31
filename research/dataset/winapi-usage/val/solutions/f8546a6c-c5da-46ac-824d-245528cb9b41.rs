#![deny(warnings)]

use std::ffi::OsString;
use std::os::windows::ffi::OsStringExt;
use windows::core::{Error, Result, PWSTR};
use windows::Win32::System::WindowsProgramming::GetComputerNameW;

pub fn get_computer_name() -> Result<String> {
    let mut buffer = vec![0u16; 256];
    let mut size = buffer.len() as u32;

    // SAFETY: buffer is properly allocated and size is initialized to its capacity.
    let success = unsafe { GetComputerNameW(Some(PWSTR(buffer.as_mut_ptr())), &mut size) };

    if success.is_ok() {
        // SAFETY: GetComputerNameW guarantees a null-terminated string within the buffer bounds.
        let len = size as usize;
        let os_str = OsString::from_wide(&buffer[..len]);
        Ok(os_str.to_string_lossy().into_owned())
    } else {
        Err(Error::from_thread())
    }
}
