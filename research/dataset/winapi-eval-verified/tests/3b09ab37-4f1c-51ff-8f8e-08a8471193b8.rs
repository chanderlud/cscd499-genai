#[cfg(test)]
mod tests {
    use super::*;
    use windows::core::{Error, HRESULT};
    use windows::Win32::Foundation::ERROR_ACCESS_DENIED;

    const E_FAIL: HRESULT = HRESULT(0x80004005u32 as i32);

    #[test]
    fn extracts_win32_code() {
        let err = Error::from_hresult(HRESULT::from_win32(ERROR_ACCESS_DENIED.0));
        assert_eq!(try_as_win32(&err), Some(ERROR_ACCESS_DENIED));
    }

    #[test]
    fn returns_none_for_non_win32_hresult() {
        let err = Error::from_hresult(E_FAIL);
        assert_eq!(try_as_win32(&err), None);
    }
}
