#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::{self, File};
    use std::io::Write;
    use std::path::Path;

    #[test]
    fn test_file_size_happy_path() {
        let temp_dir = tempfile::tempdir().unwrap();
        let file_path = temp_dir.path().join("testfile.dat");
        let content = b"Hello, world!";
        File::create(&file_path)
            .unwrap()
            .write_all(content)
            .unwrap();

        let size = file_size(&file_path).unwrap();
        assert_eq!(size, content.len() as u64);
    }

    #[test]
    fn test_file_size_empty_file() {
        let temp_dir = tempfile::tempdir().unwrap();
        let file_path = temp_dir.path().join("empty.dat");
        File::create(&file_path).unwrap();

        let size = file_size(&file_path).unwrap();
        assert_eq!(size, 0);
    }

    #[test]
    fn test_file_size_nonexistent_file() {
        let temp_dir = tempfile::tempdir().unwrap();
        let file_path = temp_dir.path().join("missing.dat");

        let result = file_size(&file_path);
        assert!(result.is_err());
    }

    #[test]
    fn test_file_size_large_file() {
        let temp_dir = tempfile::tempdir().unwrap();
        let file_path = temp_dir.path().join("large.dat");
        let large_content = vec![42u8; 1024 * 1024]; // 1MB
        File::create(&file_path)
            .unwrap()
            .write_all(&large_content)
            .unwrap();

        let size = file_size(&file_path).unwrap();
        assert_eq!(size, large_content.len() as u64);
    }

    #[test]
    fn test_file_size_directory() {
        let temp_dir = tempfile::tempdir().unwrap();
        let result = file_size(temp_dir.path());
        assert!(result.is_err());
    }
}
