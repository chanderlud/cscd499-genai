#[cfg(test)]
mod tests {
    use super::set_last_error_code;
    use windows::Win32::Foundation::{GetLastError, NO_ERROR};

    #[test]
    fn sets_nonzero_last_error_code() {
        set_last_error_code(12345);
        let actual = unsafe { GetLastError() };
        assert_eq!(actual.0, 12345);
    }

    #[test]
    fn sets_zero_last_error_code() {
        set_last_error_code(0);
        let actual = unsafe { GetLastError() };
        assert_eq!(actual, NO_ERROR);
    }
}
