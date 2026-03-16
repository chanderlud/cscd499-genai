#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::io::Read;
    use std::path::Path;

    #[test]
    fn test_write_all_empty_file() {
        let temp = tempfile::tempdir().unwrap();
        let path = temp.path().join("empty.bin");
        write_all(&path, &[]).unwrap();
        assert!(path.exists());
        assert_eq!(fs::metadata(&path).unwrap().len(), 0);
    }

    #[test]
    fn test_write_all_small_data() {
        let temp = tempfile::tempdir().unwrap();
        let path = temp.path().join("small.bin");
        write_all(&path, b"hello").unwrap();
        let content = fs::read(&path).unwrap();
        assert_eq!(content, b"hello");
    }

    #[test]
    fn test_write_all_large_data() {
        let temp = tempfile::tempdir().unwrap();
        let path = temp.path().join("large.bin");
        let data = vec![42u8; 1024 * 1024]; // 1MB of 0x2A
        write_all(&path, &data).unwrap();
        let content = fs::read(&path).unwrap();
        assert_eq!(content, data);
    }

    #[test]
    fn test_write_all_overwrite_existing() {
        let temp = tempfile::tempdir().unwrap();
        let path = temp.path().join("overwrite.bin");
        fs::write(&path, b"old content").unwrap();
        write_all(&path, b"new content").unwrap();
        let content = fs::read(&path).unwrap();
        assert_eq!(content, b"new content");
    }

    #[test]
    fn test_write_all_creates_file_in_existing_dir() {
        let temp = tempfile::tempdir().unwrap();
        let path = temp.path().join("file.bin");
        write_all(&path, b"data").unwrap();
        assert!(path.exists());
        let content = fs::read(&path).unwrap();
        assert_eq!(content, b"data");
    }

    #[test]
    fn test_write_all_fails_if_parent_dir_missing() {
        let temp = tempfile::tempdir().unwrap();
        let path = temp.path().join("nonexistent_dir").join("file.bin");
        let result = write_all(&path, b"data");
        assert!(result.is_err());
    }

    #[test]
    fn test_write_all_error_invalid_path() {
        let path = Path::new(r"\\?\INVALID\PATH\file.bin");
        let result = write_all(path, b"data");
        assert!(result.is_err());
    }
}
