#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_com_marshal_stream_roundtrip_empty() -> Result<()> {
        let result = com_marshal_stream_roundtrip(&[])?;
        assert!(result.is_empty());
        Ok(())
    }

    #[test]
    fn test_com_marshal_stream_roundtrip_small_data() -> Result<()> {
        let input = b"hello";
        let result = com_marshal_stream_roundtrip(input)?;
        assert_eq!(result.as_slice(), input);
        Ok(())
    }

    #[test]
    fn test_com_marshal_stream_roundtrip_medium_data() -> Result<()> {
        let input = b"the quick brown fox jumps over the lazy dog";
        let result = com_marshal_stream_roundtrip(input)?;
        assert_eq!(result.as_slice(), input);
        Ok(())
    }

    #[test]
    fn test_com_marshal_stream_roundtrip_large_data() -> Result<()> {
        let input = vec![42u8; 10_000];
        let result = com_marshal_stream_roundtrip(&input)?;
        assert_eq!(result, input);
        Ok(())
    }

    #[test]
    fn test_com_marshal_stream_roundtrip_non_ascii() -> Result<()> {
        let input = "こんにちは世界".as_bytes();
        let result = com_marshal_stream_roundtrip(input)?;
        assert_eq!(result.as_slice(), input);
        Ok(())
    }

    #[test]
    fn test_com_marshal_stream_roundtrip_binary_data() -> Result<()> {
        let input = vec![0u8, 255, 128, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10];
        let result = com_marshal_stream_roundtrip(&input)?;
        assert_eq!(result, input);
        Ok(())
    }

    #[test]
    fn test_com_marshal_stream_roundtrip_all_byte_values() -> Result<()> {
        let input: Vec<u8> = (0u8..=255).collect();
        let result = com_marshal_stream_roundtrip(&input)?;
        assert_eq!(result, input);
        Ok(())
    }

    #[test]
    fn test_com_marshal_stream_roundtrip_single_byte() -> Result<()> {
        let input = b"A";
        let result = com_marshal_stream_roundtrip(input)?;
        assert_eq!(result.as_slice(), input);
        Ok(())
    }

    #[test]
    fn test_com_marshal_stream_roundtrip_two_bytes() -> Result<()> {
        let input = b"Hi";
        let result = com_marshal_stream_roundtrip(input)?;
        assert_eq!(result.as_slice(), input);
        Ok(())
    }

    #[test]
    fn test_com_marshal_stream_roundtrip_multiple_calls_independent() -> Result<()> {
        let first = b"first payload";
        let second = b"second payload with different length";
        let third = b"third";

        assert_eq!(com_marshal_stream_roundtrip(first)?.as_slice(), first);
        assert_eq!(com_marshal_stream_roundtrip(second)?.as_slice(), second);
        assert_eq!(com_marshal_stream_roundtrip(third)?.as_slice(), third);

        Ok(())
    }

    #[test]
    fn test_com_marshal_stream_roundtrip_called_from_non_main_thread() -> Result<()> {
        let input = b"called from another outer thread".to_vec();
        let expected = input.clone();

        let handle = std::thread::spawn(move || {
            com_marshal_stream_roundtrip(&input).expect("roundtrip should succeed")
        });

        let result = handle.join().expect("worker thread panicked");
        assert_eq!(result, expected);
        Ok(())
    }
}
