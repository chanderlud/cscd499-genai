#[cfg(all(test, windows))]
mod tests {
    use super::*;
    use std::env;
    use std::fs;
    use std::path::{Component, Path, PathBuf};
    use std::sync::atomic::{AtomicU64, Ordering};
    use std::sync::Mutex;
    use std::time::{SystemTime, UNIX_EPOCH};

    // Tests that change the current directory must be serialized because the
    // current directory is process-global on Windows.
    static CWD_LOCK: Mutex<()> = Mutex::new(());
    static NEXT_ID: AtomicU64 = AtomicU64::new(0);

    fn make_temp_root() -> PathBuf {
        let id = NEXT_ID.fetch_add(1, Ordering::Relaxed);
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();

        let dir = env::temp_dir().join(format!(
            "full_path_tests_{}_{}_{}",
            std::process::id(),
            nanos,
            id
        ));
        fs::create_dir_all(&dir).unwrap();
        dir
    }

    fn with_temp_cwd<F: FnOnce(&Path)>(f: F) {
        let _guard = CWD_LOCK.lock().unwrap();
        let original = env::current_dir().unwrap();

        let root = make_temp_root();
        let cwd = root.join("parent").join("work");
        fs::create_dir_all(&cwd).unwrap();

        struct RestoreCwd(PathBuf);
        impl Drop for RestoreCwd {
            fn drop(&mut self) {
                let _ = env::set_current_dir(&self.0);
            }
        }

        {
            let _restore = RestoreCwd(original);
            env::set_current_dir(&cwd).unwrap();
            f(&cwd);
        }

        let _ = fs::remove_dir_all(root);
    }

    fn current_root(path: &Path) -> PathBuf {
        match path.components().next() {
            Some(Component::Prefix(prefix)) => {
                let mut root = PathBuf::from(prefix.as_os_str());
                root.push(r"\");
                root
            }
            _ => panic!("expected a Windows path with a prefix"),
        }
    }

    #[test]
    fn test_absolute_dos_path_is_unchanged() {
        let input = Path::new(r"C:\absolute\path\to\file.txt");
        let result = full_path(input).expect("full_path should succeed");
        assert!(result.is_absolute());
        assert_eq!(result, PathBuf::from(r"C:\absolute\path\to\file.txt"));
    }

    #[test]
    fn test_relative_path_with_dot() {
        with_temp_cwd(|cwd| {
            let input = Path::new(r".\data\file.txt");
            let result = full_path(input).expect("full_path should succeed");
            assert!(result.is_absolute());
            assert_eq!(result, cwd.join("data").join("file.txt"));
        });
    }

    #[test]
    fn test_relative_path_with_double_dot() {
        with_temp_cwd(|cwd| {
            let input = Path::new(r"..\data\file.txt");
            let result = full_path(input).expect("full_path should succeed");
            assert!(result.is_absolute());
            assert_eq!(
                result,
                cwd.parent().unwrap().join("data").join("file.txt")
            );
        });
    }

    #[test]
    fn test_root_relative_path() {
        with_temp_cwd(|cwd| {
            let input = Path::new(r"\data\file.txt");
            let result = full_path(input).expect("full_path should succeed");
            assert!(result.is_absolute());
            assert_eq!(result, current_root(cwd).join("data").join("file.txt"));
        });
    }

    #[test]
    fn test_nonexistent_relative_path_still_succeeds() {
        with_temp_cwd(|cwd| {
            let input = Path::new(r".\does-not-exist\file.txt");
            let result = full_path(input).expect("full_path should succeed");
            assert!(result.is_absolute());
            assert_eq!(result, cwd.join("does-not-exist").join("file.txt"));
        });
    }

    #[test]
    fn test_path_with_trailing_backslash() {
        let input = Path::new(r"C:\path\to\");
        let result = full_path(input).expect("full_path should succeed");
        assert!(result.is_absolute());
        assert_eq!(result, PathBuf::from(r"C:\path\to"));
    }

    #[test]
    fn test_path_with_mixed_separators() {
        let input = Path::new(r"C:/path/to/file.txt");
        let result = full_path(input).expect("full_path should succeed");
        assert!(result.is_absolute());
        assert_eq!(result, PathBuf::from(r"C:\path\to\file.txt"));
    }

    #[test]
    fn test_path_with_spaces() {
        let input = Path::new(r"C:\path with spaces\file.txt");
        let result = full_path(input).expect("full_path should succeed");
        assert!(result.is_absolute());
        assert_eq!(result, PathBuf::from(r"C:\path with spaces\file.txt"));
    }

    #[test]
    fn test_path_with_unicode() {
        let input = Path::new(r"C:\path\目录\file.txt");
        let result = full_path(input).expect("full_path should succeed");
        assert!(result.is_absolute());
        assert_eq!(result, PathBuf::from(r"C:\path\目录\file.txt"));
    }

    #[test]
    fn test_path_with_multiple_dots() {
        let input = Path::new(r"C:\path\.\to\.\file.txt");
        let result = full_path(input).expect("full_path should succeed");
        assert!(result.is_absolute());
        assert_eq!(result, PathBuf::from(r"C:\path\to\file.txt"));
    }

    #[test]
    fn test_path_with_multiple_double_dots() {
        let input = Path::new(r"C:\path\a\b\..\..\file.txt");
        let result = full_path(input).expect("full_path should succeed");
        assert!(result.is_absolute());
        assert_eq!(result, PathBuf::from(r"C:\path\file.txt"));
    }

    #[test]
    fn test_path_with_multiple_consecutive_backslashes() {
        let input = Path::new(r"C:\\path\\to\\file.txt");
        let result = full_path(input).expect("full_path should succeed");
        assert!(result.is_absolute());
        assert_eq!(result, PathBuf::from(r"C:\path\to\file.txt"));
    }

    #[test]
    fn test_path_with_trailing_dot() {
        let input = Path::new(r"C:\path\to\.");
        let result = full_path(input).expect("full_path should succeed");
        assert!(result.is_absolute());
        assert_eq!(result, PathBuf::from(r"C:\path\to"));
    }

    #[test]
    fn test_path_with_trailing_double_dot() {
        let input = Path::new(r"C:\path\to\..");
        let result = full_path(input).expect("full_path should succeed");
        assert!(result.is_absolute());
        assert_eq!(result, PathBuf::from(r"C:\path"));
    }

    #[test]
    fn test_path_with_unc() {
        let input = Path::new(r"\\server\share\path\file.txt");
        let result = full_path(input).expect("full_path should succeed");
        assert!(result.is_absolute());
        assert_eq!(result, PathBuf::from(r"\\server\share\path\file.txt"));
    }

    #[test]
    fn test_path_with_device_namespace() {
        let input = Path::new(r"\\.\C:\path\file.txt");
        let result = full_path(input).expect("full_path should succeed");
        assert!(result.is_absolute());
        assert_eq!(result, PathBuf::from(r"\\.\C:\path\file.txt"));
    }

    #[test]
    fn test_path_with_long_path_prefix() {
        let input = Path::new(r"\\?\C:\path\file.txt");
        let result = full_path(input).expect("full_path should succeed");
        assert!(result.is_absolute());
        assert_eq!(result, PathBuf::from(r"\\?\C:\path\file.txt"));
    }
}