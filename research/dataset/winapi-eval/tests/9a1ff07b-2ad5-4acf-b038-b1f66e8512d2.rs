#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_pidl_roundtrip_simple_file_success() {
        let path = PathBuf::from("C:\\tmp\\a.txt");
        let result = pidl_roundtrip(&path).unwrap();
        assert_eq!(result, path);
    }

    #[test]
    fn test_pidl_roundtrip_directory_success() {
        let path = PathBuf::from("C:\\tmp");
        let result = pidl_roundtrip(&path).unwrap();
        assert_eq!(result, path);
    }

    #[test]
    fn test_pidl_roundtrip_empty_path_failure() {
        let path = PathBuf::new();
        let result = pidl_roundtrip(&path);
        assert!(result.is_err());
    }

    #[test]
    fn test_pidl_roundtrip_nonexistent_path_failure() {
        let path = PathBuf::from("C:\\nonexistent\\file.txt");
        let result = pidl_roundtrip(&path);
        assert!(result.is_err());
    }

    #[test]
    fn test_pidl_roundtrip_long_path_success() {
        let path =
            PathBuf::from("C:\\tmp\\a_very_long_filename_with_many_characters_123456789.txt");
        let result = pidl_roundtrip(&path).unwrap();
        assert_eq!(result, path);
    }

    #[test]
    fn test_pidl_roundtrip_unicode_path_success() {
        let path = PathBuf::from("C:\\tmp\\测试文件.txt");
        let result = pidl_roundtrip(&path).unwrap();
        assert_eq!(result, path);
    }
}
