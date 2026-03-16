#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::path::Path;

    #[test]
    fn test_read_to_end_happy_path() {
        let temp_dir = tempfile::tempdir().unwrap();
        let file_path = temp_dir.path().join("test_file.bin");
        let content = b"Hello, World!";
        fs::write(&file_path, content).unwrap();

        let result = read_to_end(&file_path);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), content);
    }

    #[test]
    fn test_read_to_end_empty_file() {
        let temp_dir = tempfile::tempdir().unwrap();
        let file_path = temp_dir.path().join("empty_file.bin");
        fs::write(&file_path, b"").unwrap();

        let result = read_to_end(&file_path);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), b"");
    }

    #[test]
    fn test_read_to_end_nonexistent_file() {
        let temp_dir = tempfile::tempdir().unwrap();
        let file_path = temp_dir.path().join("missing_file.bin");

        let result = read_to_end(&file_path);
        assert!(result.is_err());
    }

    #[test]
    fn test_read_to_end_large_file() {
        let temp_dir = tempfile::tempdir().unwrap();
        let file_path = temp_dir.path().join("large_file.bin");
        let large_content = vec![42u8; 1024 * 1024]; // 1MB of 0x2A
        fs::write(&file_path, &large_content).unwrap();

        let result = read_to_end(&file_path);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), large_content);
    }

    #[test]
    fn test_read_to_end_special_characters_in_path() {
        let temp_dir = tempfile::tempdir().unwrap();
        let file_path = temp_dir
            .path()
            .join("test file with spaces and_unicode_😊.bin");
        let content = b"Special chars test";
        fs::write(&file_path, content).unwrap();

        let result = read_to_end(&file_path);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), content);
    }
}
