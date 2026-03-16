#[cfg(test)]
mod tests {
    use super::*;
    use windows::core::PWSTR;

    fn wide(s: &str) -> Vec<u16> {
        s.encode_utf16().chain(std::iter::once(0)).collect()
    }

    fn read_wide(buf: &[u16]) -> String {
        let end = buf.iter().position(|&x| x == 0).unwrap_or(buf.len());
        String::from_utf16(&buf[..end]).unwrap()
    }

    #[test]
    fn replaces_backslashes() {
        let mut data = wide(r"C:\tmp\a\b");
        let n = normalize_separators(PWSTR::from_raw(data.as_mut_ptr()));
        assert_eq!(n, 3);
        assert_eq!(read_wide(&data), "C:/tmp/a/b");
    }

    #[test]
    fn leaves_clean_path_alone() {
        let mut data = wide("already/clean");
        let n = normalize_separators(PWSTR::from_raw(data.as_mut_ptr()));
        assert_eq!(n, 0);
        assert_eq!(read_wide(&data), "already/clean");
    }

    #[test]
    fn null_pointer_is_noop() {
        assert_eq!(normalize_separators(PWSTR::null()), 0);
    }
}
