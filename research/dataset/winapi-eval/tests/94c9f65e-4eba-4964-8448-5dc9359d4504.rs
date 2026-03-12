#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dpapi_roundtrip_basic() {
        let data = b"hello world";
        let result = dpapi_roundtrip(data, b"pepper").unwrap();
        assert_eq!(result.as_slice(), data);
    }

    #[test]
    fn test_dpapi_roundtrip_empty_data() {
        let data = b"";
        let result = dpapi_roundtrip(data, b"pepper").unwrap();
        assert_eq!(result.as_slice(), data);
    }

    #[test]
    fn test_dpapi_roundtrip_empty_entropy() {
        let data = b"hello world";
        let result = dpapi_roundtrip(data, &[]).unwrap();
        assert_eq!(result.as_slice(), data);
    }

    #[test]
    fn test_dpapi_roundtrip_both_empty() {
        let data = b"";
        let result = dpapi_roundtrip(data, &[]).unwrap();
        assert_eq!(result.as_slice(), data);
    }

    #[test]
    fn test_dpapi_roundtrip_long_data() {
        let data = vec![42u8; 1024];
        let result = dpapi_roundtrip(&data, b"pepper").unwrap();
        assert_eq!(result, data);
    }

    #[test]
    fn test_dpapi_roundtrip_long_entropy() {
        let data = b"secret";
        let entropy = vec![99u8; 1024];
        let result = dpapi_roundtrip(data, &entropy).unwrap();
        assert_eq!(result.as_slice(), data);
    }

    #[test]
    fn test_dpapi_roundtrip_invalid_entropy() {
        let data = b"secret";
        let entropy = vec![0u8; 1024];
        let result = dpapi_roundtrip(data, &entropy).unwrap();
        assert_eq!(result.as_slice(), data);
    }

    #[test]
    fn test_dpapi_roundtrip_error_case_entropy_too_long() {
        // This test assumes the function returns an error for invalid input
        // If the function never errors, this test should be removed
        let data = b"secret";
        let entropy = vec![0u8; 1025]; // entropy too long
        let result = dpapi_roundtrip(data, &entropy);
        assert!(result.is_err());
    }
}
