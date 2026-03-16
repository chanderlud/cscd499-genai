use windows::core::Result;
use windows::Win32::Foundation::{ERROR_ALREADY_EXISTS, ERROR_FILE_EXISTS};

pub fn ok_if_already_exists(result: Result<()>) -> Result<()> {
    match result {
        Ok(()) => Ok(()),
        Err(e) => {
            // Get the HRESULT from the error
            let hr = e.code();

            // Check if it's a Win32 error (facility code 7)
            if (hr.0 as u32 >> 16) & 0x7FFF == 7 {
                // Extract the Win32 error code (lower 16 bits)
                let win32_code = hr.0 as u32 & 0xFFFF;

                if win32_code == ERROR_ALREADY_EXISTS.0 || win32_code == ERROR_FILE_EXISTS.0 {
                    return Ok(());
                }
            }
            Err(e)
        }
    }
}
