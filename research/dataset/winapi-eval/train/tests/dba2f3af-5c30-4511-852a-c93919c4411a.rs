#[cfg(test)]
#[cfg(target_os = "windows")]
mod tests {
    use super::*;
    use std::fs;
    use std::path::Path;

    #[test]
    fn test_property_size_valid_file() {
        // Create a temporary file with known size
        let temp_dir = tempfile::tempdir().unwrap();
        let file_path = temp_dir.path().join("testfile.bin");
        let content = vec![0u8; 1024];
        fs::write(&file_path, &content).unwrap();

        let size = property_size(&file_path).unwrap();
        assert_eq!(size, 1024);
    }

    #[test]
    fn test_property_size_empty_file() {
        let temp_dir = tempfile::tempdir().unwrap();
        let file_path = temp_dir.path().join("empty.bin");
        fs::write(&file_path, &[]).unwrap();

        let size = property_size(&file_path).unwrap();
        assert_eq!(size, 0);
    }

    #[test]
    fn test_property_size_nonexistent_file() {
        let temp_dir = tempfile::tempdir().unwrap();
        let file_path = temp_dir.path().join("missing.bin");

        let result = property_size(&file_path);
        assert!(result.is_err());
    }

    #[test]
    fn test_property_size_directory() {
        let temp_dir = tempfile::tempdir().unwrap();
        let result = property_size(temp_dir.path());
        assert!(result.is_err());
    }

    #[test]
    fn test_property_size_large_file() {
        let temp_dir = tempfile::tempdir().unwrap();
        let file_path = temp_dir.path().join("largefile.bin");
        let content = vec![0u8; 10 * 1024 * 1024]; // 10 MB
        fs::write(&file_path, &content).unwrap();

        let size = property_size(&file_path).unwrap();
        assert_eq!(size, 10 * 1024 * 1024);
    }
}
