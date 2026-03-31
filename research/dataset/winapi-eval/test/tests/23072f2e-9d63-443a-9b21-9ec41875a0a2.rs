#[cfg(test)]
mod tests {
    use super::rename_file;
    use std::fs;
    use std::path::PathBuf;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn unique_temp_file(name: &str) -> PathBuf {
        let stamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();

        std::env::temp_dir().join(format!("{name}_{stamp}.txt"))
    }

    #[test]
    fn renames_file_and_preserves_contents() {
        let from = unique_temp_file("movefilew_from");
        let to = unique_temp_file("movefilew_to");

        let _ = fs::remove_file(&from);
        let _ = fs::remove_file(&to);

        fs::write(&from, b"hello windows").unwrap();

        rename_file(&from, &to).unwrap();

        assert!(!from.exists());
        assert!(to.exists());
        assert_eq!(fs::read(&to).unwrap(), b"hello windows");

        fs::remove_file(&to).unwrap();
    }

    #[test]
    fn returns_error_when_source_does_not_exist() {
        let from = unique_temp_file("movefilew_missing_from");
        let to = unique_temp_file("movefilew_missing_to");

        let _ = fs::remove_file(&from);
        let _ = fs::remove_file(&to);

        let result = rename_file(&from, &to);

        assert!(result.is_err());
        assert!(!to.exists());
    }
}
