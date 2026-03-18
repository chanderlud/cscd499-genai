#[cfg(test)]
#[cfg(target_os = "windows")]
mod tests {
    use super::*;
    use std::fs;
    use std::path::{Path, PathBuf};
    use tempfile;

    // Helper to normalize paths by removing \\?\ prefix for comparison
    fn normalize_path(path: &Path) -> String {
        let s = path.to_string_lossy().to_string();
        if s.starts_with("\\\\?\\") {
            s[4..].to_string()
        } else {
            s
        }
    }

    #[test]
    fn test_final_path_existing_file() {
        let temp_file = tempfile::NamedTempFile::new().unwrap();
        let expected = fs::canonicalize(temp_file.path()).unwrap();
        let result = final_path(temp_file.path()).unwrap();
        assert_eq!(normalize_path(&result), normalize_path(&expected));
    }

    #[test]
    fn test_final_path_existing_directory() {
        let temp_dir = tempfile::tempdir().unwrap();
        let expected = fs::canonicalize(temp_dir.path()).unwrap();
        let result = final_path(temp_dir.path()).unwrap();
        assert_eq!(normalize_path(&result), normalize_path(&expected));
    }

    #[test]
    fn test_final_path_long_path() {
        // Create a path longer than MAX_PATH (260 chars)
        let base = PathBuf::from(r"\\?\C:\");
        let long_path = base.join("a".repeat(200)).join("b.txt");
        fs::create_dir_all(long_path.parent().unwrap()).unwrap();
        fs::write(&long_path, b"test").unwrap();

        let expected = fs::canonicalize(&long_path).unwrap();
        let result = final_path(&long_path).unwrap();
        assert_eq!(normalize_path(&result), normalize_path(&expected));
    }

    #[test]
    fn test_final_path_with_junctions() {
        let temp_dir = tempfile::tempdir().unwrap();
        let target = temp_dir.path().join("target");
        let link = temp_dir.path().join("link");
        fs::create_dir(&target).unwrap();
        std::os::windows::fs::symlink_dir(&target, &link).unwrap();

        let expected = fs::canonicalize(&target).unwrap();
        let result = final_path(&link).unwrap();
        assert_eq!(normalize_path(&result), normalize_path(&expected));
    }

    #[test]
    fn test_final_path_nonexistent() {
        let non_existent = Path::new(r"C:\nonexistent\file.txt");
        assert!(final_path(non_existent).is_err());
    }

    #[test]
    fn test_final_path_empty_path() {
        let empty_path = Path::new("");
        assert!(final_path(empty_path).is_err());
    }

    #[test]
    fn test_final_path_relative_path() {
        // Create a temporary file and use a relative path to it
        let temp_file = tempfile::NamedTempFile::new().unwrap();
        let file_name = temp_file.path().file_name().unwrap();
        let parent = temp_file.path().parent().unwrap();

        // Change to the parent directory and use relative path
        let original_dir = std::env::current_dir().unwrap();
        std::env::set_current_dir(parent).unwrap();

        let relative = Path::new(file_name);
        let expected = fs::canonicalize(temp_file.path()).unwrap();
        let result = final_path(relative).unwrap();

        // Restore original directory
        std::env::set_current_dir(original_dir).unwrap();

        assert_eq!(normalize_path(&result), normalize_path(&expected));
    }
}
