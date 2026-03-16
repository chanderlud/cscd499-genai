#[cfg(test)]
mod tests {
    use super::*;
    use windows::core::HRESULT;

    const E_FAIL: HRESULT = HRESULT(0x80004005u32 as i32);

    #[test]
    fn s_ok_is_success() {
        assert!(hresult_to_result(HRESULT(0)).is_ok());
    }

    #[test]
    fn s_false_is_also_success() {
        assert!(hresult_to_result(HRESULT(1)).is_ok());
    }

    #[test]
    fn failing_hresult_becomes_error() {
        let err = hresult_to_result(E_FAIL).unwrap_err();
        assert_eq!(err.code(), E_FAIL);
    }
}
