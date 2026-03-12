#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::path::Path;

    #[test]
    fn test_append_all_creates_file_when_missing() {
        let temp_dir = tempfile::tempdir().unwrap();
        let path = temp_dir.path().join("new_file.txt");
        append_all(&path, b"hello").unwrap();
        assert!(path.exists());
        assert_eq!(fs::read(&path).unwrap(), b"hello");
    }

    #[test]
    fn test_append_all_appends_to_existing_file() {
        let temp_dir = tempfile::tempdir().unwrap();
        let path = temp_dir.path().join("existing.txt");
        fs::write(&path, b"start").unwrap();
        append_all(&path, b"middle").unwrap();
        append_all(&path, b"end").unwrap();
        assert_eq!(fs::read(&path).unwrap(), b"startmiddleend");
    }

    #[test]
    fn test_append_all_empty_data_does_not_truncate() {
        let temp_dir = tempfile::tempdir().unwrap();
        let path = temp_dir.path().join("file.txt");
        fs::write(&path, b"content").unwrap();
        append_all(&path, b"").unwrap();
        assert_eq!(fs::read(&path).unwrap(), b"content");
    }

    #[test]
    fn test_append_all_empty_file() {
        let temp_dir = tempfile::tempdir().unwrap();
        let path = temp_dir.path().join("empty.txt");
        fs::write(&path, b"").unwrap();
        append_all(&path, b"data").unwrap();
        assert_eq!(fs::read(&path).unwrap(), b"data");
    }

    #[test]
    fn test_append_all_large_data() {
        let temp_dir = tempfile::tempdir().unwrap();
        let path = temp_dir.path().join("large.txt");
        let large_data = vec![42u8; 100_000];
        append_all(&path, &large_data).unwrap();
        assert_eq!(fs::read(&path).unwrap(), large_data);
    }

    #[test]
    fn test_append_all_invalid_path_fails() {
        let invalid_path = Path::new("/invalid/path/that/does/not/exist/file.txt");
        let result = append_all(invalid_path, b"data");
        assert!(result.is_err());
    }
}
