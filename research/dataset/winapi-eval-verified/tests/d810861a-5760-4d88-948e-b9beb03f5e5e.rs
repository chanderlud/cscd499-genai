#[cfg(test)]
mod tests {
    use super::*;
    use windows::core::{PCWSTR, Result};

    fn wide(s: &str) -> Vec<u16> {
        s.encode_utf16().chain(std::iter::once(0)).collect()
    }

    #[test]
    fn converts_ascii_and_unicode() {
        let data = wide("héllo");
        let out = pcwstr_to_string(PCWSTR::from_raw(data.as_ptr())).unwrap();
        assert_eq!(out, "héllo");
    }

    #[test]
    fn null_pointer_becomes_empty_string() {
        let out = pcwstr_to_string(PCWSTR::null()).unwrap();
        assert_eq!(out, "");
    }

    #[test]
    fn invalid_utf16_returns_error() {
        let data = [0xD800u16, 0];
        assert!(pcwstr_to_string(PCWSTR::from_raw(data.as_ptr())).is_err());
    }
}
