#[cfg(test)]
mod tests {
    use super::make_file_read_only;
    use std::fs;
    use std::path::PathBuf;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn unique_temp_file_path(prefix: &str) -> PathBuf {
        let mut path = std::env::temp_dir();
        let stamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        path.push(format!("{prefix}_{stamp}.txt"));
        path
    }

    #[test]
    fn marks_existing_file_as_read_only() {
        let path = unique_temp_file_path("make_file_read_only");
        fs::write(&path, b"hello").unwrap();

        make_file_read_only(path.to_str().unwrap()).unwrap();

        let metadata = fs::metadata(&path).unwrap();
        assert!(metadata.permissions().readonly());

        let mut perms = metadata.permissions();
        perms.set_readonly(false);
        fs::set_permissions(&path, perms).unwrap();
        fs::remove_file(&path).unwrap();
    }

    #[test]
    fn returns_error_for_missing_file() {
        let path = unique_temp_file_path("make_file_read_only_missing");

        let result = make_file_read_only(path.to_str().unwrap());

        assert!(result.is_err());
    }
}
