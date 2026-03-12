#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::path::Path;

    #[test]
    fn test_stg_stream_roundtrip_basic() -> Result<()> {
        let temp_dir = tempfile::tempdir()?;
        let path = temp_dir.path().join("test_basic.stg");
        let data = b"hello world";
        let out = stg_stream_roundtrip(&path, "Data", data)?;
        assert_eq!(out.as_slice(), data);
        Ok(())
    }

    #[test]
    fn test_stg_stream_roundtrip_empty() -> Result<()> {
        let temp_dir = tempfile::tempdir()?;
        let path = temp_dir.path().join("test_empty.stg");
        let empty: &[u8] = &[];
        let out = stg_stream_roundtrip(&path, "Empty", empty)?;
        assert!(out.is_empty());
        Ok(())
    }

    #[test]
    fn test_stg_stream_roundtrip_large() -> Result<()> {
        let temp_dir = tempfile::tempdir()?;
        let path = temp_dir.path().join("test_large.stg");
        let data = vec![42u8; 1024 * 1024]; // 1MB
        let out = stg_stream_roundtrip(&path, "Large", &data)?;
        assert_eq!(out.as_slice(), data);
        Ok(())
    }

    #[test]
    fn test_stg_stream_roundtrip_special_chars() -> Result<()> {
        let temp_dir = tempfile::tempdir()?;
        let path = temp_dir.path().join("test_special.stg");
        let data = b"special \x00 \n \t \xFF";
        let out = stg_stream_roundtrip(&path, "Special", data)?;
        assert_eq!(out.as_slice(), data);
        Ok(())
    }

    #[test]
    fn test_stg_stream_roundtrip_reopen() -> Result<()> {
        let temp_dir = tempfile::tempdir()?;
        let path = temp_dir.path().join("test_reopen.stg");
        let data = b"roundtrip";
        let out1 = stg_stream_roundtrip(&path, "Data", data)?;
        assert_eq!(out1.as_slice(), data);

        // Reopen and read again to verify persistence
        let out2 = stg_stream_roundtrip(&path, "Data", data)?;
        assert_eq!(out2.as_slice(), data);
        Ok(())
    }

    #[test]
    fn test_stg_stream_roundtrip_different_stream() -> Result<()> {
        let temp_dir = tempfile::tempdir()?;
        let path = temp_dir.path().join("test_multi.stg");
        let data1 = b"first stream";
        let data2 = b"second stream";
        let out1 = stg_stream_roundtrip(&path, "Stream1", data1)?;
        let out2 = stg_stream_roundtrip(&path, "Stream2", data2)?;
        assert_eq!(out1.as_slice(), data1);
        assert_eq!(out2.as_slice(), data2);
        Ok(())
    }
}
