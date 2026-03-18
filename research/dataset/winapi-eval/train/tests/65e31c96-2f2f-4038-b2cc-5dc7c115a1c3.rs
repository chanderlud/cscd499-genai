#[cfg(test)]
mod tests {
    use super::*;
    use windows::core::{PWSTR, Result};

    fn read_wide(buf: &[u16]) -> String {
        let end = buf.iter().position(|&x| x == 0).unwrap_or(buf.len());
        String::from_utf16(&buf[..end]).unwrap()
    }

    #[test]
    fn writes_basic_string() {
        let mut buf = vec![0u16; 8];
        let n = write_wide(PWSTR::from_raw(buf.as_mut_ptr()), buf.len(), "cat").unwrap();
        assert_eq!(n, 3);
        assert_eq!(read_wide(&buf), "cat");
    }

    #[test]
    fn allows_exact_fit() {
        let mut buf = vec![0u16; 4];
        let n = write_wide(PWSTR::from_raw(buf.as_mut_ptr()), buf.len(), "hey").unwrap();
        assert_eq!(n, 3);
        assert_eq!(read_wide(&buf), "hey");
    }

    #[test]
    fn rejects_too_small_buffer() {
        let mut buf = vec![0u16; 3];
        assert!(write_wide(PWSTR::from_raw(buf.as_mut_ptr()), buf.len(), "tool").is_err());
    }

    #[test]
    fn writes_empty_string() {
        let mut buf = vec![99u16; 1];
        let n = write_wide(PWSTR::from_raw(buf.as_mut_ptr()), buf.len(), "").unwrap();
        assert_eq!(n, 0);
        assert_eq!(buf[0], 0);
    }
}
