#[cfg(test)]
mod tests {
    use super::*;
    use windows::core::HRESULT;
    use windows::Win32::Foundation::{ERROR_ACCESS_DENIED, WIN32_ERROR};

    #[test]
    fn converts_zero_to_success() {
        assert_eq!(win32_to_hresult(WIN32_ERROR(0)), HRESULT(0));
    }

    #[test]
    fn converts_nonzero_error() {
        assert_eq!(
            win32_to_hresult(ERROR_ACCESS_DENIED),
            HRESULT::from_win32(ERROR_ACCESS_DENIED.0)
        );
    }
}
