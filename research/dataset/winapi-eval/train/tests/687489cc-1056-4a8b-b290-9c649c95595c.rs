#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::io::{self, Read};
    use tempfile::tempdir;

    #[test]
    fn test_atomic_write_creates_file_with_correct_content() -> Result<()> {
        let temp_dir = tempdir()?;
        let dest_path = temp_dir.path().join("test.json");
        let data = br#"{"key":"value"}"#;
        atomic_write(&dest_path, data)?;
        let mut file = fs::File::open(&dest_path)?;
        let mut contents = String::new();
        file.read_to_string(&mut contents)?;
        assert_eq!(contents, r#"{"key":"value"}"#);
        Ok(())
    }

    #[test]
    fn test_atomic_write_overwrites_existing_file() -> Result<()> {
        let temp_dir = tempdir()?;
        let dest_path = temp_dir.path().join("test.json");
        fs::write(&dest_path, "old content")?;
        let data = br#"{"key":"value"}"#;
        atomic_write(&dest_path, data)?;
        let contents = fs::read_to_string(&dest_path)?;
        assert_eq!(contents, r#"{"key":"value"}"#);
        Ok(())
    }

    #[test]
    fn test_atomic_write_empty_content() -> Result<()> {
        let temp_dir = tempdir()?;
        let dest_path = temp_dir.path().join("empty.json");
        let data = b""; // empty slice
        atomic_write(&dest_path, data)?;
        let contents = fs::read(&dest_path)?;
        assert!(contents.is_empty());
        Ok(())
    }

    #[test]
    fn test_atomic_write_nonexistent_directory() -> Result<()> {
        let dest_path = Path::new(r"\\nonexistent\path\file.json");
        let result = atomic_write(dest_path, b"data");
        assert!(result.is_err());
        Ok(())
    }

    #[test]
    fn test_atomic_write_with_long_content() -> Result<()> {
        let temp_dir = tempdir()?;
        let dest_path = temp_dir.path().join("long.json");
        let data = vec![b'a'; 10_000]; // 10KB of 'a'
        atomic_write(&dest_path, &data)?;
        let contents = fs::read(&dest_path)?;
        assert_eq!(contents.len(), 10_000);
        Ok(())
    }

    #[test]
    fn test_atomic_write_no_temp_file_left() -> Result<()> {
        let temp_dir = tempdir()?;
        let dest_path = temp_dir.path().join("test.json");
        let data = br#"{"key":"value"}"#;
        atomic_write(&dest_path, data)?;
        // Verify that the directory only contains the destination file
        let entries: Vec<_> = fs::read_dir(temp_dir.path())?
            .filter_map(|e| e.ok())
            .collect();
        assert_eq!(
            entries.len(),
            1,
            "Expected exactly one file in the directory"
        );
        assert_eq!(entries[0].file_name(), "test.json");
        Ok(())
    }
}
