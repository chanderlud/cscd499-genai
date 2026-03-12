#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;
    use std::path::PathBuf;

    #[test]
    fn test_final_path_existing_file() {
        // Create a temporary file and get its final path
        let temp_file = tempfile::NamedTempFile::new().unwrap();
        let final_path = final_path(temp_file.path()).unwrap();

        // Should be absolute and match the temp file's actual path
        assert!(final_path.is_absolute());
        assert_eq!(final_path, temp_file.path());
    }

    #[test]
    fn test_final_path_existing_directory() {
        // Test with a directory
        let temp_dir = tempfile::tempdir().unwrap();
        let final_path = final_path(temp_dir.path()).unwrap();

        assert!(final_path.is_absolute());
        assert_eq!(final_path, temp_dir.path());
    }

    #[test]
    fn test_final_path_long_path() {
        // Test with a long path (over 260 chars)
        let long_path = PathBuf::from(r"\\?\C:\")
            .join("a".repeat(100))
            .join("b.txt");
        let _ = std::fs::write(&long_path, b"test").unwrap();

        let final_path = final_path(&long_path).unwrap();
        assert!(final_path.is_absolute());
        assert_eq!(final_path, long_path);
    }

    #[test]
    fn test_final_path_with_junctions() {
        // Test with a junction point (symlink on Windows)
        let temp_dir = tempfile::tempdir().unwrap();
        let target = temp_dir.path().join("target");
        let link = temp_dir.path().join("link");
        std::fs::create_dir(&target).unwrap();
        std::os::windows::fs::symlink_dir(&target, &link).unwrap();

        let final_path = final_path(&link).unwrap();
        assert!(final_path.is_absolute());
        assert_eq!(final_path, target);
    }

    #[test]
    fn test_final_path_nonexistent() {
        // Non-existent file should return an error
        let non_existent = Path::new(r"C:\nonexistent\file.txt");
        let err = final_path(non_existent).unwrap_err();
        assert!(err.to_string().contains("invalid path"));
    }

    #[test]
    fn test_final_path_empty_path() {
        // Empty path should return an error
        let empty_path = Path::new("");
        let err = final_path(empty_path).unwrap_err();
        assert!(err.to_string().contains("invalid path"));
    }

    #[test]
    fn test_final_path_relative_path() {
        // Relative path should be normalized to absolute
        let relative = Path::new("test.txt");
        let final_path = final_path(relative).unwrap();

        assert!(final_path.is_absolute());
        assert!(final_path.ends_with("test.txt"));
    }
}
