#[cfg(test)]
mod tests {
    use super::*;
    use windows::core::{Error, HRESULT};
    use windows::Win32::Foundation::{ERROR_ACCESS_DENIED, ERROR_FILE_NOT_FOUND};

    #[test]
    fn matches_same_raw_os_error() {
        let win = Error::from_hresult(HRESULT::from_win32(ERROR_ACCESS_DENIED.0));
        let io = std::io::Error::from_raw_os_error(ERROR_ACCESS_DENIED.0 as i32);
        assert!(same_os_error(&win, &io));
    }

    #[test]
    fn rejects_different_errors() {
        let win = Error::from_hresult(HRESULT::from_win32(ERROR_ACCESS_DENIED.0));
        let io = std::io::Error::from_raw_os_error(ERROR_FILE_NOT_FOUND.0 as i32);
        assert!(!same_os_error(&win, &io));
    }

    #[test]
    fn rejects_io_errors_without_raw_code() {
        let win = Error::from_hresult(HRESULT::from_win32(ERROR_ACCESS_DENIED.0));
        let io = std::io::Error::new(std::io::ErrorKind::Other, "boom");
        assert!(!same_os_error(&win, &io));
    }
}
