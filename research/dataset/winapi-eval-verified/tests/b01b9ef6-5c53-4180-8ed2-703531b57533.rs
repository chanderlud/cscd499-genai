#[cfg(test)]
mod tests {
    use super::*;
    use sha2::{Digest, Sha256};

    #[test]
    fn test_sspi_ntlm_seal_roundtrip_basic() -> Result<()> {
        let plaintext = b"Hello, world!";
        let roundtripped = sspi_ntlm_seal_roundtrip(plaintext)?;
        assert_eq!(roundtripped.as_slice(), plaintext);
        Ok(())
    }

    #[test]
    fn test_sspi_ntlm_seal_roundtrip_empty() -> Result<()> {
        let plaintext = b"";
        let roundtripped = sspi_ntlm_seal_roundtrip(plaintext)?;
        assert_eq!(roundtripped.as_slice(), plaintext);
        Ok(())
    }

    #[test]
    fn test_sspi_ntlm_seal_roundtrip_small() -> Result<()> {
        let plaintext = b"A";
        let roundtripped = sspi_ntlm_seal_roundtrip(plaintext)?;
        assert_eq!(roundtripped.as_slice(), plaintext);
        Ok(())
    }

    #[test]
    fn test_sspi_ntlm_seal_roundtrip_large() -> Result<()> {
        let plaintext = vec![42u8; 10_000];
        let roundtripped = sspi_ntlm_seal_roundtrip(&plaintext)?;
        assert_eq!(roundtripped.as_slice(), plaintext);
        Ok(())
    }

    #[test]
    fn test_sspi_ntlm_seal_roundtrip_non_ascii() -> Result<()> {
        let plaintext = "こんにちは世界".as_bytes();
        let roundtripped = sspi_ntlm_seal_roundtrip(plaintext)?;
        assert_eq!(roundtripped.as_slice(), plaintext);
        Ok(())
    }

    #[test]
    fn test_sspi_ntlm_seal_roundtrip_repeated() -> Result<()> {
        let plaintext = b"secrets, but corporate";
        let roundtripped1 = sspi_ntlm_seal_roundtrip(plaintext)?;
        let roundtripped2 = sspi_ntlm_seal_roundtrip(plaintext)?;
        assert_eq!(roundtripped1.as_slice(), plaintext);
        assert_eq!(roundtripped2.as_slice(), plaintext);
        Ok(())
    }

    #[test]
    fn test_sspi_ntlm_seal_roundtrip_hash_consistency() -> Result<()> {
        let plaintext = b"test data";
        let roundtripped = sspi_ntlm_seal_roundtrip(plaintext)?;
        let mut hasher = Sha256::new();
        hasher.update(&roundtripped);
        let hash = hasher.finalize();
        assert_eq!(hash.as_slice(), Sha256::digest(plaintext).as_slice());
        Ok(())
    }
}
