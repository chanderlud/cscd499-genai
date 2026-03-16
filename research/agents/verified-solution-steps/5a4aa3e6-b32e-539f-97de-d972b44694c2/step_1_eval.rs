use windows::core::{Error, HRESULT, E_FAIL};

pub fn io_error_to_windows(err: std::io::Error) -> Error {
    match err.raw_os_error() {
        Some(code) => {
            // Convert i32 OS error code to u32 for HRESULT::from_win32
            let hresult = HRESULT::from_win32(code as u32);
            Error::from_hresult(hresult)
        }
        None => Error::from_hresult(E_FAIL),
    }
}