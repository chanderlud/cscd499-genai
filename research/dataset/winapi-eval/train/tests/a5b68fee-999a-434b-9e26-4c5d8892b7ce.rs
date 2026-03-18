#[cfg(test)]
mod tests {
    use super::*;
    use windows::core::PCWSTR;

    fn wide(s: &str) -> Vec<u16> {
        s.encode_utf16().chain(std::iter::once(0)).collect()
    }

    #[test]
    fn matches_exact_string() {
        let data = wide("rust");
        assert!(pcwstr_eq(PCWSTR::from_raw(data.as_ptr()), "rust"));
    }

    #[test]
    fn detects_mismatch() {
        let data = wide("rust");
        assert!(!pcwstr_eq(PCWSTR::from_raw(data.as_ptr()), "Rust"));
    }

    #[test]
    fn null_pointer_only_matches_empty() {
        assert!(pcwstr_eq(PCWSTR::null(), ""));
        assert!(!pcwstr_eq(PCWSTR::null(), "x"));
    }
}
