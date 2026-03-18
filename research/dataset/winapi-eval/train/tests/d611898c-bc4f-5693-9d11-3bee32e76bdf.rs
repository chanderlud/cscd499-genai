#[cfg(test)]
mod tests {
    use super::*;
    use windows::core::{Error, HRESULT};
    use windows::Win32::Foundation::{ERROR_ACCESS_DENIED, ERROR_FILE_NOT_FOUND};

    #[test]
    fn matches_access_denied() {
        let err = Error::from_hresult(HRESULT::from_win32(ERROR_ACCESS_DENIED.0));
        assert!(is_access_denied(&err));
    }

    #[test]
    fn rejects_other_win32_errors() {
        let err = Error::from_hresult(HRESULT::from_win32(ERROR_FILE_NOT_FOUND.0));
        assert!(!is_access_denied(&err));
    }
}
