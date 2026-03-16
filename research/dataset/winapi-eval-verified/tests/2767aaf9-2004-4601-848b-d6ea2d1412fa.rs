#[cfg(test)]
mod tests {
    use super::*;
    use windows::core::{PCWSTR, Result};

    fn wide(s: &str) -> Vec<u16> {
        s.encode_utf16().chain(std::iter::once(0)).collect()
    }

    #[test]
    fn collects_multiple_strings() {
        let a = wide("one");
        let b = wide("two");
        let out = collect_wide_strings(&[
            PCWSTR::from_raw(a.as_ptr()),
            PCWSTR::from_raw(b.as_ptr()),
        ])
        .unwrap();
        assert_eq!(out, vec!["one", "two"]);
    }

    #[test]
    fn empty_slice_returns_empty_vec() {
        let out = collect_wide_strings(&[]).unwrap();
        assert!(out.is_empty());
    }

    #[test]
    fn null_pointer_in_slice_is_error() {
        let a = wide("ok");
        assert!(collect_wide_strings(&[
            PCWSTR::from_raw(a.as_ptr()),
            PCWSTR::null(),
        ])
        .is_err());
    }

    #[test]
    fn invalid_utf16_in_slice_is_error() {
        let bad = [0xD800u16, 0];
        assert!(collect_wide_strings(&[PCWSTR::from_raw(bad.as_ptr())]).is_err());
    }
}
