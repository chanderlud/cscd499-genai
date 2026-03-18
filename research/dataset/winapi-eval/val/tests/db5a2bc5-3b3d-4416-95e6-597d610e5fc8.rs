#[cfg(test)]
mod tests {
    use windows::core::PCWSTR;
    use super::*;

    fn wide(s: &str) -> Vec<u16> {
        s.encode_utf16().chain(std::iter::once(0)).collect()
    }

    #[test]
    fn counts_basic_string() {
        let data = wide("hello");
        assert_eq!(pcwstr_len(PCWSTR::from_raw(data.as_ptr())), 5);
    }

    #[test]
    fn stops_at_first_nul() {
        let data = [b'a' as u16, 0, b'b' as u16, 0];
        assert_eq!(pcwstr_len(PCWSTR::from_raw(data.as_ptr())), 1);
    }

    #[test]
    fn null_pointer_is_zero() {
        assert_eq!(pcwstr_len(PCWSTR::null()), 0);
    }
}
