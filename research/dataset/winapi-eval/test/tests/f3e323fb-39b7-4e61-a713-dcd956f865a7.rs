#[cfg(test)]
mod tests {
    use super::delete_file;
    use std::fs;
    use std::path::PathBuf;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn unique_temp_file(prefix: &str) -> PathBuf {
        let stamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();

        std::env::temp_dir().join(format!("{prefix}_{stamp}.txt"))
    }

    #[test]
    fn deletes_an_existing_file() {
        let path = unique_temp_file("delete_file_test");
        let _ = fs::remove_file(&path);

        fs::write(&path, b"hello").unwrap();
        assert!(path.exists());

        delete_file(&path).unwrap();

        assert!(!path.exists());
    }

    #[test]
    fn returns_error_for_missing_file() {
        let path = unique_temp_file("delete_file_missing");
        let _ = fs::remove_file(&path);

        let result = delete_file(&path);

        assert!(result.is_err());
        assert!(!path.exists());
    }
}
