use windows::core::{Error, Result, HRESULT};
use windows::Win32::Foundation::{ERROR_ACCESS_DENIED, ERROR_FILE_NOT_FOUND, WIN32_ERROR};

pub fn find_matching_error_pair(
    win_errors: &[windows::core::Error],
    io_errors: &[std::io::Error],
) -> Option<(usize, usize)> {
    for (i, win_err) in win_errors.iter().enumerate() {
        // Get the HRESULT from the Windows error
        let hr = win_err.code();

        // Check if it's a Win32 error (facility code 7)
        if hr.0 & 0x7FFF == 7 {
            // Extract the Win32 error code (lower 16 bits)
            let win32_code = (hr.0 & 0xFFFF) as u32;

            for (j, io_err) in io_errors.iter().enumerate() {
                // Check if the IO error has an OS error code
                if let Some(io_code) = io_err.raw_os_error() {
                    // Compare the error codes
                    if win32_code == io_code as u32 {
                        return Some((i, j));
                    }
                }
            }
        }
    }

    None
}
