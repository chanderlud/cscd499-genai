#[cfg(test)]
mod tests {
    use super::*;
    use windows::core::HRESULT;
    use windows::Win32::Foundation::{ERROR_ACCESS_DENIED, WIN32_ERROR};

    #[test]
    fn success_code_becomes_ok() {
        assert!(check_win32(WIN32_ERROR(0)).is_ok());
    }

    #[test]
    fn error_code_becomes_err() {
        let err = check_win32(ERROR_ACCESS_DENIED).unwrap_err();
        assert_eq!(err.code(), HRESULT::from_win32(ERROR_ACCESS_DENIED.0));
    }
}
