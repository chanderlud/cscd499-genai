#[cfg(test)]
mod tests {
    use super::*;
    use std::io;

    #[test]
    fn test_pipe_checksum_basic() -> io::Result<()> {
        let data = [1, 2, 3, 4];
        let checksum = pipe_checksum(&data)?;
        assert_eq!(checksum, 10);
        Ok(())
    }

    #[test]
    fn test_pipe_checksum_empty() -> io::Result<()> {
        let data: [u8; 0] = [];
        let checksum = pipe_checksum(&data)?;
        assert_eq!(checksum, 0);
        Ok(())
    }

    #[test]
    fn test_pipe_checksum_single_byte() -> io::Result<()> {
        let data = [42];
        let checksum = pipe_checksum(&data)?;
        assert_eq!(checksum, 42);
        Ok(())
    }

    #[test]
    fn test_pipe_checksum_large_data() -> io::Result<()> {
        let data: Vec<u8> = (0..255).collect();
        let expected: u32 = data.iter().map(|&b| b as u32).sum();
        let checksum = pipe_checksum(&data)?;
        assert_eq!(checksum, expected);
        Ok(())
    }

    #[test]
    fn test_pipe_checksum_modulo_wrap() -> io::Result<()> {
        let data = [0xFF, 0xFF, 0xFF, 0xFF];
        let checksum = pipe_checksum(&data)?;
        assert_eq!(checksum, 0xFFFFFFFC);
        Ok(())
    }

    #[test]
    fn test_pipe_checksum_non_ascii() -> io::Result<()> {
        let data = [0xAB, 0xCD, 0xEF];
        let checksum = pipe_checksum(&data)?;
        assert_eq!(checksum, 0x2A9);
        Ok(())
    }

    #[test]
    fn test_pipe_checksum_all_zeros() -> io::Result<()> {
        let data = [0; 100];
        let checksum = pipe_checksum(&data)?;
        assert_eq!(checksum, 0);
        Ok(())
    }

    #[test]
    fn test_pipe_checksum_mixed_values() -> io::Result<()> {
        let data = [0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08];
        let checksum = pipe_checksum(&data)?;
        assert_eq!(checksum, 36);
        Ok(())
    }
}
