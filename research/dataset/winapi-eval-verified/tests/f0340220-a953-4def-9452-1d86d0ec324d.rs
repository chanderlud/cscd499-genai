#[cfg(test)]
mod tests {
    use super::*;
    use sha2::{Digest, Sha256};

    #[test]
    fn test_sha256_empty() {
        let result = sha256(&[]).unwrap();
        let expected: [u8; 32] = Sha256::digest(&[]).into();
        assert_eq!(result, expected);
    }

    #[test]
    fn test_sha256_known() {
        let input = b"abc";
        let expected: [u8; 32] = Sha256::digest(input).into();
        let result = sha256(input).unwrap();
        assert_eq!(result, expected);
    }

    #[test]
    fn test_sha256_multiple_blocks() {
        let input = vec![0x61; 64 * 3]; // "aaa..." spanning multiple SHA-256 blocks
        let expected: [u8; 32] = Sha256::digest(&input).into();
        let result = sha256(&input).unwrap();
        assert_eq!(result, expected);
    }
}
