use windows::core::{Error, Result, PWSTR};
use windows::Win32::System::WindowsProgramming::GetComputerNameW;

pub fn get_computer_name() -> Result<String> {
    let mut buffer = [0u16; 256];
    let mut size = buffer.len() as u32;

    // SAFETY: GetComputerNameW expects a valid mutable pointer to a u16 buffer
    // and a pointer to a u32 for the buffer size. We provide both correctly.
    unsafe {
        GetComputerNameW(Some(PWSTR(buffer.as_mut_ptr())), &mut size)?;
    }

    Ok(String::from_utf16_lossy(&buffer[..size as usize]))
}
