#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::path::{Path, PathBuf};
    use std::time::{SystemTime, UNIX_EPOCH};

    fn unique_stem(prefix: &str) -> String {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();

        format!("{prefix}_{}_{}", std::process::id(), nanos)
    }

    fn cleanup_file(path: &Path) {
        if !path.exists() {
            return;
        }

        if let Ok(metadata) = fs::metadata(path) {
            let mut perms = metadata.permissions();
            perms.set_readonly(false);
            let _ = fs::set_permissions(path, perms);
        }

        let _ = fs::remove_file(path);
    }

    struct CleanupGuard(PathBuf);

    impl Drop for CleanupGuard {
        fn drop(&mut self) {
            cleanup_file(&self.0);
        }
    }

    #[test]
    fn writes_bytes_and_marks_file_readonly() {
        let stem = unique_stem("write_readonly_temp_file");
        let result = write_readonly_temp_file(&stem, b"hello from win32").unwrap();
        let _guard = CleanupGuard(result.path.clone());

        let bytes = fs::read(&result.path).unwrap();
        let metadata = fs::metadata(&result.path).unwrap();

        assert_eq!(bytes, b"hello from win32");
        assert_eq!(result.bytes_written, 16);
        assert!(metadata.permissions().readonly());
        assert_eq!(
            result.path.file_name().unwrap().to_string_lossy(),
            format!("{stem}.txt")
        );
    }

    #[test]
    fn supports_empty_contents() {
        let stem = unique_stem("write_readonly_temp_file_empty");
        let result = write_readonly_temp_file(&stem, b"").unwrap();
        let _guard = CleanupGuard(result.path.clone());

        let metadata = fs::metadata(&result.path).unwrap();

        assert_eq!(metadata.len(), 0);
        assert_eq!(result.bytes_written, 0);
        assert!(metadata.permissions().readonly());
    }
}
