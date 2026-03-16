#[cfg(test)]
mod tests {
    use super::*;
    use windows::core::HRESULT;
    use windows::Win32::Foundation::ERROR_ACCESS_DENIED;

    const E_FAIL: HRESULT = HRESULT(0x80004005u32 as i32);

    #[test]
    fn raw_os_error_is_preserved() {
        let io = std::io::Error::from_raw_os_error(ERROR_ACCESS_DENIED.0 as i32);
        let err = io_error_to_windows(io);
        assert_eq!(err.code(), HRESULT::from_win32(ERROR_ACCESS_DENIED.0));
    }

    #[test]
    fn missing_raw_os_error_becomes_efail() {
        let io = std::io::Error::new(std::io::ErrorKind::Other, "boom");
        let err = io_error_to_windows(io);
        assert_eq!(err.code(), E_FAIL);
    }
}
