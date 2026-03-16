#[cfg(test)]
mod tests {
    use super::*;
    use windows::core::{Error, HRESULT};
    use windows::Win32::Foundation::{
        ERROR_ACCESS_DENIED, ERROR_ALREADY_EXISTS, ERROR_FILE_EXISTS,
    };

    #[test]
    fn ok_stays_ok() {
        assert!(ok_if_already_exists(Ok(())).is_ok());
    }

    #[test]
    fn already_exists_is_ignored() {
        let r = Err(Error::from_hresult(HRESULT::from_win32(ERROR_ALREADY_EXISTS.0)));
        assert!(ok_if_already_exists(r).is_ok());
    }

    #[test]
    fn file_exists_is_ignored() {
        let r = Err(Error::from_hresult(HRESULT::from_win32(ERROR_FILE_EXISTS.0)));
        assert!(ok_if_already_exists(r).is_ok());
    }

    #[test]
    fn unrelated_error_is_preserved() {
        let r = Err(Error::from_hresult(HRESULT::from_win32(ERROR_ACCESS_DENIED.0)));
        let err = ok_if_already_exists(r).unwrap_err();
        assert_eq!(err.code(), HRESULT::from_win32(ERROR_ACCESS_DENIED.0));
    }
}
