#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;
    use std::sync::atomic::{AtomicUsize, Ordering};

    static COUNTER: AtomicUsize = AtomicUsize::new(0);

    fn create_temp_dir(test_name: &str) -> PathBuf {
        let temp_dir = std::env::temp_dir();
        loop {
            let unique = COUNTER.fetch_add(1, Ordering::SeqCst);
            let path = temp_dir.join(format!("{}_{}", test_name, unique));
            if std::fs::create_dir(&path).is_ok() {
                return path;
            }
        }
    }

    #[test]
    fn test_pidl_roundtrip_simple_file_success() {
        let temp_dir = create_temp_dir("simple_file");
        let file_path = temp_dir.join("a.txt");
        std::fs::write(&file_path, b"test").unwrap();

        let result = pidl_roundtrip(&file_path).unwrap();
        assert_eq!(result, file_path);

        std::fs::remove_dir_all(&temp_dir).unwrap();
    }

    #[test]
    fn test_pidl_roundtrip_directory_success() {
        let temp_dir = create_temp_dir("directory");
        let dir_path = temp_dir.join("subdir");
        std::fs::create_dir(&dir_path).unwrap();

        let result = pidl_roundtrip(&dir_path).unwrap();
        assert_eq!(result, dir_path);

        std::fs::remove_dir_all(&temp_dir).unwrap();
    }

    #[test]
    fn test_pidl_roundtrip_empty_path_failure() {
        let path = PathBuf::new();
        let result = pidl_roundtrip(&path);
        assert!(result.is_err());
    }

    #[test]
    fn test_pidl_roundtrip_nonexistent_path_failure() {
        let temp_dir = create_temp_dir("nonexistent");
        let file_path = temp_dir.join("nonexistent.txt");

        let result = pidl_roundtrip(&file_path);
        assert!(result.is_err());

        std::fs::remove_dir_all(&temp_dir).unwrap();
    }

    #[test]
    fn test_pidl_roundtrip_long_path_success() {
        let temp_dir = create_temp_dir("long_path");
        let file_name = "a_very_long_filename_with_many_characters_123456789.txt";
        let file_path = temp_dir.join(file_name);
        std::fs::write(&file_path, b"test").unwrap();

        let result = pidl_roundtrip(&file_path).unwrap();
        assert_eq!(result, file_path);

        std::fs::remove_dir_all(&temp_dir).unwrap();
    }

    #[test]
    fn test_pidl_roundtrip_unicode_path_success() {
        let temp_dir = create_temp_dir("unicode_path");
        let file_path = temp_dir.join("测试文件.txt");
        std::fs::write(&file_path, b"test").unwrap();

        let result = pidl_roundtrip(&file_path).unwrap();
        assert_eq!(result, file_path);

        std::fs::remove_dir_all(&temp_dir).unwrap();
    }
}