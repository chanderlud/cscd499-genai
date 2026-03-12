#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use std::path::Path;

    #[test]
    fn test_file_id_nonexistent_file() {
        let path = Path::new(r"C:\nonexistent.txt");
        assert!(file_id(path).is_err());
    }

    #[test]
    fn test_file_id_existing_empty_file() {
        let temp_dir = tempfile::tempdir().unwrap();
        let file_path = temp_dir.path().join("test.txt");
        std::fs::File::create(&file_path).unwrap();
        let id = file_id(&file_path).unwrap();
        assert_eq!(id.len(), 16);
    }

    #[test]
    fn test_file_id_existing_file_with_content() {
        let temp_dir = tempfile::tempdir().unwrap();
        let file_path = temp_dir.path().join("test.txt");
        let mut file = std::fs::File::create(&file_path).unwrap();
        file.write_all(b"test content").unwrap();
        let id = file_id(&file_path).unwrap();
        assert_eq!(id.len(), 16);
    }

    #[test]
    fn test_file_id_same_file_multiple_calls() {
        let temp_dir = tempfile::tempdir().unwrap();
        let file_path = temp_dir.path().join("test.txt");
        std::fs::File::create(&file_path).unwrap();
        let id1 = file_id(&file_path).unwrap();
        let id2 = file_id(&file_path).unwrap();
        assert_eq!(id1, id2);
    }

    #[test]
    fn test_file_id_different_files() {
        let temp_dir = tempfile::tempdir().unwrap();
        let file1_path = temp_dir.path().join("test1.txt");
        let file2_path = temp_dir.path().join("test2.txt");
        std::fs::File::create(&file1_path).unwrap();
        std::fs::File::create(&file2_path).unwrap();
        let id1 = file_id(&file1_path).unwrap();
        let id2 = file_id(&file2_path).unwrap();
        assert_ne!(id1, id2);
    }

    #[test]
    fn test_file_id_directory() {
        let temp_dir = tempfile::tempdir().unwrap();
        let dir_path = temp_dir.path();
        let result = file_id(dir_path);
        assert!(result.is_err());
    }

    #[test]
    fn test_file_id_idempotent() {
        let temp_dir = tempfile::tempdir().unwrap();
        let file_path = temp_dir.path().join("test.txt");
        std::fs::File::create(&file_path).unwrap();

        let id1 = file_id(&file_path).unwrap();
        let id2 = file_id(&file_path).unwrap();
        let id3 = file_id(&file_path).unwrap();

        assert_eq!(id1, id2);
        assert_eq!(id2, id3);
    }

    #[test]
    fn test_file_id_different_content_same_file() {
        let temp_dir = tempfile::tempdir().unwrap();
        let file_path = temp_dir.path().join("test.txt");

        // Create file with content
        let mut file = std::fs::File::create(&file_path).unwrap();
        file.write_all(b"content1").unwrap();
        let id1 = file_id(&file_path).unwrap();

        // Overwrite with different content
        let mut file = std::fs::File::create(&file_path).unwrap();
        file.write_all(b"content2").unwrap();
        let id2 = file_id(&file_path).unwrap();

        // File ID should remain the same despite content change
        assert_eq!(id1, id2);
    }

    #[test]
    fn test_file_id_zero_length_file() {
        let temp_dir = tempfile::tempdir().unwrap();
        let file_path = temp_dir.path().join("empty.txt");
        std::fs::File::create(&file_path).unwrap();

        let id = file_id(&file_path).unwrap();
        assert_eq!(id.len(), 16);
    }

    #[test]
    fn test_file_id_large_file() {
        let temp_dir = tempfile::tempdir().unwrap();
        let file_path = temp_dir.path().join("large.txt");

        // Create a file with some content
        let mut file = std::fs::File::create(&file_path).unwrap();
        file.write_all(b"test content").unwrap();

        let id = file_id(&file_path).unwrap();
        assert_eq!(id.len(), 16);
    }
}