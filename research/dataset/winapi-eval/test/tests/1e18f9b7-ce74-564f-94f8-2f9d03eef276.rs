#[cfg(test)]
mod tests {
    use super::*;
    use windows::core::{Error, HRESULT};
    use windows::Win32::Foundation::{
        ERROR_ACCESS_DENIED, ERROR_FILE_NOT_FOUND, ERROR_PATH_NOT_FOUND,
    };

    #[test]
    fn success_means_exists() {
        assert_eq!(exists_from_result(Ok(())).unwrap(), true);
    }

    #[test]
    fn file_not_found_means_false() {
        let r = Err(Error::from_hresult(HRESULT::from_win32(ERROR_FILE_NOT_FOUND.0)));
        assert_eq!(exists_from_result(r).unwrap(), false);
    }

    #[test]
    fn path_not_found_means_false() {
        let r = Err(Error::from_hresult(HRESULT::from_win32(ERROR_PATH_NOT_FOUND.0)));
        assert_eq!(exists_from_result(r).unwrap(), false);
    }

    #[test]
    fn unrelated_error_propagates() {
        let r = Err(Error::from_hresult(HRESULT::from_win32(ERROR_ACCESS_DENIED.0)));
        let err = exists_from_result(r).unwrap_err();
        assert_eq!(err.code(), HRESULT::from_win32(ERROR_ACCESS_DENIED.0));
    }
}
