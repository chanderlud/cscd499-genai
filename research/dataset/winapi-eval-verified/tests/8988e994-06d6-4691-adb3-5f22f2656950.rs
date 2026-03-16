#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::path::PathBuf;
    use std::time::{SystemTime, UNIX_EPOCH};
    use windows::Win32::System::Com::{CoInitializeEx, CoUninitialize, COINIT_APARTMENTTHREADED};

    struct ComGuard;

    impl ComGuard {
        fn new() -> Self {
            unsafe {
                _ = CoInitializeEx(None, COINIT_APARTMENTTHREADED);
            }
            Self
        }
    }

    impl Drop for ComGuard {
        fn drop(&mut self) {
            unsafe {
                CoUninitialize();
            }
        }
    }

    fn unique_test_path(name: &str) -> PathBuf {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();

        std::env::temp_dir().join(format!(
            "shellitem_test_{}_{}_{}",
            std::process::id(),
            name,
            nonce
        ))
    }

    #[test]
    fn shellitem_filesystem_path_returns_ok_for_existing_file() {
        let _com = ComGuard::new();

        let test_path = unique_test_path("file.txt");
        fs::write(&test_path, b"hello").unwrap();

        let result = shellitem_filesystem_path(&test_path);

        fs::remove_file(&test_path).unwrap();

        assert!(
            result.is_ok(),
            "Expected Ok result for existing file, got: {:?}",
            result
        );

        let path = result.unwrap();
        assert!(!path.as_os_str().is_empty(), "Result path should not be empty");
        assert!(path.is_absolute(), "Result path should be absolute");
    }

    #[test]
    fn shellitem_filesystem_path_returns_ok_for_existing_directory() {
        let _com = ComGuard::new();

        let test_path = unique_test_path("dir");
        fs::create_dir(&test_path).unwrap();

        let result = shellitem_filesystem_path(&test_path);

        fs::remove_dir(&test_path).unwrap();

        assert!(
            result.is_ok(),
            "Expected Ok result for existing directory, got: {:?}",
            result
        );

        let path = result.unwrap();
        assert!(!path.as_os_str().is_empty(), "Result path should not be empty");
        assert!(path.is_absolute(), "Result path should be absolute");
    }

    #[test]
    fn shellitem_filesystem_path_returns_err_for_nonexistent_path() {
        let _com = ComGuard::new();

        let test_path = unique_test_path("missing.txt");
        let result = shellitem_filesystem_path(&test_path);

        assert!(
            result.is_err(),
            "Expected Err result for nonexistent path, got: {:?}",
            result
        );
    }

    #[test]
    fn shellitem_filesystem_path_returns_err_for_empty_path() {
        let _com = ComGuard::new();

        let test_path = PathBuf::new();
        let result = shellitem_filesystem_path(&test_path);

        assert!(
            result.is_err(),
            "Expected Err result for empty path, got: {:?}",
            result
        );
    }

    #[test]
    fn shellitem_filesystem_path_returns_err_for_relative_path() {
        let _com = ComGuard::new();

        let test_path = PathBuf::from("relative\\path.txt");
        let result = shellitem_filesystem_path(&test_path);

        assert!(
            result.is_err(),
            "Expected Err result for relative path, got: {:?}",
            result
        );
    }

    #[test]
    fn shellitem_filesystem_path_returns_ok_for_valid_absolute_path() {
        let _com = ComGuard::new();

        let test_path = std::env::temp_dir();
        let result = shellitem_filesystem_path(&test_path);

        assert!(
            result.is_ok(),
            "Expected Ok result for valid absolute path, got: {:?}",
            result
        );

        let path = result.unwrap();
        assert!(!path.as_os_str().is_empty(), "Result path should not be empty");
        assert!(path.is_absolute(), "Result path should be absolute");
    }
}