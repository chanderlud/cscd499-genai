use windows::core::{Result, HRESULT};
use windows::Win32::Foundation::{ERROR_FILE_NOT_FOUND, ERROR_PATH_NOT_FOUND};

pub fn exists_from_result(result: Result<()>) -> Result<bool> {
    match result {
        Ok(()) => Ok(true),
        Err(e) => {
            let hr = e.code();
            if hr == HRESULT::from_win32(ERROR_FILE_NOT_FOUND.0)
                || hr == HRESULT::from_win32(ERROR_PATH_NOT_FOUND.0)
            {
                Ok(false)
            } else {
                Err(e)
            }
        }
    }
}
