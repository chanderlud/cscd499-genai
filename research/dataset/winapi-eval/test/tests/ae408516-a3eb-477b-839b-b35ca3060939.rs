#[cfg(test)]
mod tests {
    use super::change_current_directory;
    use std::fs;
    use std::path::{Path, PathBuf};
    use std::sync::{Mutex, OnceLock};
    use std::time::{SystemTime, UNIX_EPOCH};

    fn cwd_lock() -> &'static Mutex<()> {
        static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
        LOCK.get_or_init(|| Mutex::new(()))
    }

    fn unique_name(prefix: &str) -> String {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        format!("{prefix}_{nanos}")
    }

    fn canonical(path: impl AsRef<Path>) -> PathBuf {
        fs::canonicalize(path).unwrap()
    }

    struct RestoreCurrentDir(PathBuf);

    impl Drop for RestoreCurrentDir {
        fn drop(&mut self) {
            let _ = std::env::set_current_dir(&self.0);
        }
    }

    #[test]
    fn changes_the_process_current_directory() {
        let _guard = cwd_lock().lock().unwrap();

        let original = std::env::current_dir().unwrap();
        let _restore = RestoreCurrentDir(original.clone());

        let dir = std::env::temp_dir().join(unique_name("setcwd_target"));
        fs::create_dir_all(&dir).unwrap();

        change_current_directory(&dir).unwrap();

        assert_eq!(canonical(std::env::current_dir().unwrap()), canonical(&dir));

        std::env::set_current_dir(&original).unwrap();
        fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn returns_error_for_missing_directory_and_leaves_cwd_unchanged() {
        let _guard = cwd_lock().lock().unwrap();

        let original = std::env::current_dir().unwrap();
        let _restore = RestoreCurrentDir(original.clone());
        let original_canonical = canonical(&original);

        let missing = std::env::temp_dir().join(unique_name("setcwd_missing"));
        assert!(!missing.exists());

        let result = change_current_directory(&missing);

        assert!(result.is_err());
        assert_eq!(
            canonical(std::env::current_dir().unwrap()),
            original_canonical
        );
    }
}
