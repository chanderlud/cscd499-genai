#[cfg(test)]
mod tests {
    use super::*;
    use windows::core::PCSTR;

    #[test]
    fn counts_basic_ascii() {
        let data = b"hello\0";
        assert_eq!(pcstr_len(PCSTR::from_raw(data.as_ptr())), 5);
    }

    #[test]
    fn stops_at_first_nul() {
        let data = b"a\0b\0";
        assert_eq!(pcstr_len(PCSTR::from_raw(data.as_ptr())), 1);
    }

    #[test]
    fn null_pointer_is_zero() {
        assert_eq!(pcstr_len(PCSTR::null()), 0);
    }
}
