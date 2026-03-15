use windows::core::{Error, Result};
use windows::Win32::Globalization::GetUserDefaultLocaleName;

fn get_system_locale() -> Result<String> {
    // SAFETY: GetUserDefaultLocaleName is a valid Windows API call
    unsafe {
        let mut buffer = [0u16; 85];
        let len = GetUserDefaultLocaleName(&mut buffer);

        if len > 0 {
            // Convert UTF-16 buffer to String, excluding null terminator
            let locale = String::from_utf16_lossy(&buffer[..(len as usize - 1)]);
            Ok(locale)
        } else {
            // GetLastError() is captured automatically by from_thread()
            Err(Error::from_thread())
        }
    }
}

fn main() -> Result<()> {
    let locale = get_system_locale()?;
    println!("System locale: {}", locale);
    Ok(())
}
