#[cfg(test)]
mod tests {
    use super::*;
    use windows::Win32::Storage::Compression::COMPRESS_ALGORITHM_XPRESS_HUFF;

    #[test]
    fn test_compress_roundtrip_basic() -> Result<()> {
        let data = b"Hello, world!";
        let decompressed = compress_roundtrip(COMPRESS_ALGORITHM_XPRESS_HUFF.0, data)?;
        assert_eq!(decompressed, data);
        Ok(())
    }

    #[test]
    fn test_compress_roundtrip_empty() -> Result<()> {
        let data: &[u8] = &[];
        let decompressed = compress_roundtrip(COMPRESS_ALGORITHM_XPRESS_HUFF.0, data)?;
        assert_eq!(decompressed, data);
        Ok(())
    }

    #[test]
    fn test_compress_roundtrip_small_repeated() -> Result<()> {
        let data = b"aaaaabbbbcccc";
        let decompressed = compress_roundtrip(COMPRESS_ALGORITHM_XPRESS_HUFF.0, data)?;
        assert_eq!(decompressed, data);
        Ok(())
    }

    #[test]
    fn test_compress_roundtrip_large() -> Result<()> {
        let data = vec![42u8; 10_000];
        let decompressed = compress_roundtrip(COMPRESS_ALGORITHM_XPRESS_HUFF.0, &data)?;
        assert_eq!(decompressed, data);
        Ok(())
    }

    #[test]
    fn test_compress_roundtrip_invalid_algorithm() -> Result<()> {
        let data = b"test";
        let result = compress_roundtrip(0xFFFF_FFFF, data);
        assert!(result.is_err());
        Ok(())
    }

    #[test]
    fn test_compress_roundtrip_single_byte() -> Result<()> {
        let data = b"X";
        let decompressed = compress_roundtrip(COMPRESS_ALGORITHM_XPRESS_HUFF.0, data)?;
        assert_eq!(decompressed, data);
        Ok(())
    }

    #[test]
    fn test_compress_roundtrip_ascii_only() -> Result<()> {
        let data = b"Hello, world! 1234567890!@#$%^&*()_+-=[]{}|;':\",./<>?";
        let decompressed = compress_roundtrip(COMPRESS_ALGORITHM_XPRESS_HUFF.0, data)?;
        assert_eq!(decompressed, data);
        Ok(())
    }

    #[test]
    fn test_compress_roundtrip_binary() -> Result<()> {
        let data = vec![0u8; 256];
        let decompressed = compress_roundtrip(COMPRESS_ALGORITHM_XPRESS_HUFF.0, &data)?;
        assert_eq!(decompressed, data);
        Ok(())
    }

    #[test]
    fn test_compress_roundtrip_mixed_content() -> Result<()> {
        let data = b"1234567890!@#$%^&*()_+-=[]{}|;':\",./<>?";
        let decompressed = compress_roundtrip(COMPRESS_ALGORITHM_XPRESS_HUFF.0, data)?;
        assert_eq!(decompressed, data);
        Ok(())
    }

    #[test]
    fn test_compress_roundtrip_zero_bytes() -> Result<()> {
        let data = vec![0u8; 0];
        let decompressed = compress_roundtrip(COMPRESS_ALGORITHM_XPRESS_HUFF.0, &data)?;
        assert_eq!(decompressed, data);
        Ok(())
    }

    #[test]
    fn test_compress_roundtrip_very_small() -> Result<()> {
        let data = b"a";
        let decompressed = compress_roundtrip(COMPRESS_ALGORITHM_XPRESS_HUFF.0, data)?;
        assert_eq!(decompressed, data);
        Ok(())
    }

    #[test]
    fn test_compress_roundtrip_pattern() -> Result<()> {
        let data = b"abcabcabcabcabcabcabcabcabcabc";
        let decompressed = compress_roundtrip(COMPRESS_ALGORITHM_XPRESS_HUFF.0, data)?;
        assert_eq!(decompressed, data);
        Ok(())
    }
}
