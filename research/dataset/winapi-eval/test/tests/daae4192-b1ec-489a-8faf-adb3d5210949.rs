mod tests {
    use windows::core::{PCWSTR, Result};
    use super::*;

    fn wide(s: &str) -> Vec<u16> {
        s.encode_utf16().chain(std::iter::once(0)).collect()
    }

    #[test]
    fn primary_wins_when_non_empty() {
        let primary = wide("admin");
        let fallback = wide("guest");
        let out = choose_wide(
            Some(PCWSTR::from_raw(primary.as_ptr())),
            Some(PCWSTR::from_raw(fallback.as_ptr())),
        )
        .unwrap();
        assert_eq!(out, "admin");
    }

    #[test]
    fn fallback_is_used_when_primary_is_empty() {
        let primary = wide("");
        let fallback = wide("guest");
        let out = choose_wide(
            Some(PCWSTR::from_raw(primary.as_ptr())),
            Some(PCWSTR::from_raw(fallback.as_ptr())),
        )
        .unwrap();
        assert_eq!(out, "guest");
    }

    #[test]
    fn both_missing_yield_empty_string() {
        let out = choose_wide(None, Some(PCWSTR::null())).unwrap();
        assert_eq!(out, "");
    }

    #[test]
    fn invalid_selected_value_returns_error() {
        let bad = [0xD800u16, 0];
        assert!(choose_wide(Some(PCWSTR::from_raw(bad.as_ptr())), None).is_err());
    }
}
