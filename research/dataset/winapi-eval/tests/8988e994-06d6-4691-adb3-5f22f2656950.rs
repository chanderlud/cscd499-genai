#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn shellitem_filesystem_path_returns_ok_for_existing_file() {
        let test_path = PathBuf::from(r"C:\tmp\a.txt");
        let result = shellitem_filesystem_path(&test_path);
        assert!(result.is_ok(), "Expected Ok result for existing file");
        let path = result.unwrap();
        assert!(
            !path.as_os_str().is_empty(),
            "Result path should not be empty"
        );
    }

    #[test]
    fn shellitem_filesystem_path_returns_ok_for_existing_directory() {
        let test_path = PathBuf::from(r"C:\tmp");
        let result = shellitem_filesystem_path(&test_path);
        assert!(result.is_ok(), "Expected Ok result for existing directory");
        let path = result.unwrap();
        assert!(
            !path.as_os_str().is_empty(),
            "Result path should not be empty"
        );
    }

    #[test]
    fn shellitem_filesystem_path_returns_err_for_nonexistent_path() {
        let test_path = PathBuf::from(r"C:\nonexistent\file.txt");
        let result = shellitem_filesystem_path(&test_path);
        assert!(result.is_err(), "Expected Err result for nonexistent path");
    }

    #[test]
    fn shellitem_filesystem_path_returns_err_for_empty_path() {
        let test_path = PathBuf::new();
        let result = shellitem_filesystem_path(&test_path);
        assert!(result.is_err(), "Expected Err result for empty path");
    }

    #[test]
    fn shellitem_filesystem_path_returns_err_for_relative_path() {
        let test_path = PathBuf::from("relative\\path.txt");
        let result = shellitem_filesystem_path(&test_path);
        assert!(result.is_err(), "Expected Err result for relative path");
    }

    #[test]
    fn shellitem_filesystem_path_returns_ok_for_valid_absolute_path() {
        let test_path = PathBuf::from(r"C:\Windows\System32");
        let result = shellitem_filesystem_path(&test_path);
        assert!(result.is_ok(), "Expected Ok result for valid absolute path");
        let path = result.unwrap();
        assert!(
            !path.as_os_str().is_empty(),
            "Result path should not be empty"
        );
        assert!(path.is_absolute(), "Result path should be absolute");
    }
}
