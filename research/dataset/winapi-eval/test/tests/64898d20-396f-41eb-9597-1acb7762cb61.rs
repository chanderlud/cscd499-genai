#[cfg(test)]
mod tests {
    use super::create_directory;
    use std::fs;
    use std::path::PathBuf;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn unique_test_dir(prefix: &str) -> PathBuf {
        let stamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();

        std::env::temp_dir().join(format!("{prefix}_{stamp}"))
    }

    #[test]
    fn creates_directory_successfully() {
        let dir = unique_test_dir("create_directory_success");
        let dir_str = dir.to_string_lossy().into_owned();

        create_directory(&dir_str).unwrap();

        assert!(dir.exists());
        assert!(dir.is_dir());

        fs::remove_dir(&dir).unwrap();
    }

    #[test]
    fn fails_when_parent_directory_is_missing() {
        let missing_parent = unique_test_dir("missing_parent");
        let child = missing_parent.join("child");
        let child_str = child.to_string_lossy().into_owned();

        let result = create_directory(&child_str);

        assert!(result.is_err());
        assert!(!child.exists());
    }
}
