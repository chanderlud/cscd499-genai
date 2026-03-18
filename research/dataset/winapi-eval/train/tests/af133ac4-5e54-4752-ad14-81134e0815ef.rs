#[cfg(test)]
mod tests {
    use super::*;
    use windows::core::{PSTR, Result};

    fn read_bytes(buf: &[u8]) -> String {
        let end = buf.iter().position(|&x| x == 0).unwrap_or(buf.len());
        String::from_utf8(buf[..end].to_vec()).unwrap()
    }

    #[test]
    fn writes_uppercase_ascii() {
        let mut buf = vec![0u8; 8];
        let n = write_ascii_upper(PSTR::from_raw(buf.as_mut_ptr()), buf.len(), "abZ9").unwrap();
        assert_eq!(n, 4);
        assert_eq!(read_bytes(&buf), "ABZ9");
    }

    #[test]
    fn rejects_non_ascii_input() {
        let mut buf = vec![0u8; 8];
        assert!(write_ascii_upper(PSTR::from_raw(buf.as_mut_ptr()), buf.len(), "hé").is_err());
    }

    #[test]
    fn rejects_too_small_buffer() {
        let mut buf = vec![0u8; 3];
        assert!(write_ascii_upper(PSTR::from_raw(buf.as_mut_ptr()), buf.len(), "abcd").is_err());
    }

    #[test]
    fn writes_empty_string() {
        let mut buf = vec![1u8; 1];
        let n = write_ascii_upper(PSTR::from_raw(buf.as_mut_ptr()), buf.len(), "").unwrap();
        assert_eq!(n, 0);
        assert_eq!(buf[0], 0);
    }
}
