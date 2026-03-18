#[cfg(test)]
mod tests {
    use super::*;
    use sha2::{Digest as Sha2Digest, Sha256};

    #[test]
    fn test_crypt32_sign_verify_roundtrip_simple() -> Result<()> {
        let original = b"Hello, world!";
        let signed = crypt32_sign_verify_roundtrip(original)?;
        assert_eq!(signed.as_slice(), original);
        Ok(())
    }

    #[test]
    fn test_crypt32_sign_verify_roundtrip_empty() -> Result<()> {
        let original: &[u8] = b"";
        let signed = crypt32_sign_verify_roundtrip(original)?;
        assert_eq!(signed.as_slice(), original);
        Ok(())
    }

    #[test]
    fn test_crypt32_sign_verify_roundtrip_large() -> Result<()> {
        let original = vec![0x55u8; 1024 * 1024];
        let signed = crypt32_sign_verify_roundtrip(&original)?;
        assert_eq!(signed, original);
        Ok(())
    }

    #[test]
    fn test_crypt32_sign_verify_roundtrip_binary_data() -> Result<()> {
        let original = vec![0x00, 0xFF, 0xAB, 0x12, 0x34, 0x56, 0x78, 0x9A];
        let signed = crypt32_sign_verify_roundtrip(&original)?;
        assert_eq!(signed, original);
        Ok(())
    }

    #[test]
    fn test_crypt32_sign_verify_roundtrip_sha256_consistency() -> Result<()> {
        let original = b"Test message with special chars: !@#$%^&*()";
        let signed = crypt32_sign_verify_roundtrip(original)?;
        assert_eq!(signed.as_slice(), original);

        let mut hasher = Sha256::new();
        hasher.update(original);
        let original_hash = hasher.finalize();

        let mut hasher = Sha256::new();
        hasher.update(&signed);
        let signed_hash = hasher.finalize();

        assert_eq!(original_hash, signed_hash);
        Ok(())
    }
}
