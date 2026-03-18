#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    #[test]
    fn test_winhttp_get_from_local_server_basic() -> Result<()> {
        let body = b"hello world";
        let response = winhttp_get_from_local_server(body)?;
        assert_eq!(response, body);
        Ok(())
    }

    #[test]
    fn test_winhttp_get_from_local_server_empty_body() -> Result<()> {
        let response = winhttp_get_from_local_server(&[])?;
        assert!(response.is_empty());
        Ok(())
    }

    #[test]
    fn test_winhttp_get_from_local_server_long_body() -> Result<()> {
        let body = vec![42u8; 10_000];
        let response = winhttp_get_from_local_server(&body)?;
        assert_eq!(response, body);
        Ok(())
    }

    #[test]
    fn test_winhttp_get_from_local_server_non_ascii() -> Result<()> {
        let body = b"\xe2\x9c\x85\xe2\x9d\xa4";
        let response = winhttp_get_from_local_server(body)?;
        assert_eq!(response, body);
        Ok(())
    }

    #[test]
    fn test_winhttp_get_from_local_server_binary_data() -> Result<()> {
        let body = vec![
            0xFF, 0x00, 0xAB, 0x12, 0x34, 0x56, 0x78, 0x9A, 0xBC, 0xDE, 0xF0,
        ];
        let response = winhttp_get_from_local_server(&body)?;
        assert_eq!(response, body);
        Ok(())
    }
}
