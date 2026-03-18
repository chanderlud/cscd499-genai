#[cfg(test)]
mod tests {
    use super::*;
    use windows::core::HRESULT;
    use windows::Win32::Foundation::ERROR_FILE_NOT_FOUND;

    #[test]
    fn ok_value_passes_through() {
        let r: std::result::Result<u32, _> = Ok(7);
        assert_eq!(lift_win32(r).unwrap(), 7);
    }

    #[test]
    fn err_value_becomes_windows_error() {
        let r: std::result::Result<u32, _> = Err(ERROR_FILE_NOT_FOUND);
        let err = lift_win32(r).unwrap_err();
        assert_eq!(err.code(), HRESULT::from_win32(ERROR_FILE_NOT_FOUND.0));
    }
}
