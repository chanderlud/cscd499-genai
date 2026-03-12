#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hglobal_stream_roundtrip_empty() {
        let result = hglobal_stream_roundtrip(&[]).unwrap();
        assert_eq!(result, Vec::<u8>::new());
    }

    #[test]
    fn test_hglobal_stream_roundtrip_small_data() {
        let input = b"hello";
        let result = hglobal_stream_roundtrip(input).unwrap();
        assert_eq!(result, input.to_vec());
    }

    #[test]
    fn test_hglobal_stream_roundtrip_medium_data() {
        let input = vec![42u8; 1024];
        let result = hglobal_stream_roundtrip(&input).unwrap();
        assert_eq!(result, input);
    }

    #[test]
    fn test_hglobal_stream_roundtrip_large_data() {
        let input = vec![99u8; 65536];
        let result = hglobal_stream_roundtrip(&input).unwrap();
        assert_eq!(result, input);
    }

    #[test]
    fn test_hglobal_stream_roundtrip_non_ascii() {
        let input = b"\xff\xfe\xfd\xfc";
        let result = hglobal_stream_roundtrip(input).unwrap();
        assert_eq!(result, input.to_vec());
    }

    #[test]
    fn test_hglobal_stream_roundtrip_error_invalid_input() {
        // This test assumes the function should handle null pointers gracefully
        // If the function is expected to panic or return an error for null input,
        // adjust the assertion accordingly.
        let result = hglobal_stream_roundtrip(&[0u8; 0]);
        assert!(result.is_ok());
    }
}
