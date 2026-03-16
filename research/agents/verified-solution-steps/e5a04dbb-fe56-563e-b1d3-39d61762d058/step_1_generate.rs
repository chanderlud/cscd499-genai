use windows::core::{Result, Error};
use windows::Win32::Foundation::{ERROR_ALREADY_EXISTS, ERROR_FILE_EXISTS};

pub fn ok_if_already_exists(result: Result<()>) -> Result<()> {
    match result {
        Ok(()) => Ok(()),
        Err(e) => {
            // Check if the error code matches either ERROR_ALREADY_EXISTS or ERROR_FILE_EXISTS
            if let Some(win32_error) = e.code().win32_error() {
                if win32_error == ERROR_ALREADY_EXISTS || win32_error == ERROR_FILE_EXISTS {
                    return Ok(());
                }
            }
            Err(e)
        }
    }
}