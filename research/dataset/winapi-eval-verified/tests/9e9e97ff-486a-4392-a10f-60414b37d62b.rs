#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::path::Path;

    #[test]
    fn test_set_get_file_sddl_happy_path() {
        let temp_dir = tempfile::tempdir().unwrap();
        let path = temp_dir.path().join("testfile.txt");
        fs::write(&path, b"test").unwrap();

        let sddl = "D:P(A;;GA;;;SY)(A;;GA;;;BA)";
        let result = set_get_file_sddl(&path, sddl).unwrap();

        assert_eq!(result, sddl);
    }

    #[test]
    fn test_set_get_file_sddl_empty_sddl() {
        let temp_dir = tempfile::tempdir().unwrap();
        let path = temp_dir.path().join("testfile.txt");
        fs::write(&path, b"test").unwrap();

        let sddl = "";
        let result = set_get_file_sddl(&path, sddl);

        assert!(result.is_err());
    }

    #[test]
    fn test_set_get_file_sddl_invalid_sddl() {
        let temp_dir = tempfile::tempdir().unwrap();
        let path = temp_dir.path().join("testfile.txt");
        fs::write(&path, b"test").unwrap();

        let sddl = "INVALID_SDDL";
        let result = set_get_file_sddl(&path, sddl);

        assert!(result.is_err());
    }

    #[test]
    fn test_set_get_file_sddl_nonexistent_file() {
        let temp_dir = tempfile::tempdir().unwrap();
        let path = temp_dir.path().join("nonexistent.txt");

        let sddl = "D:P(A;;GA;;;SY)(A;;GA;;;BA)";
        let result = set_get_file_sddl(&path, sddl);

        assert!(result.is_err());
    }

    #[test]
    fn test_set_get_file_sddl_readonly_file() {
        let temp_dir = tempfile::tempdir().unwrap();
        let path = temp_dir.path().join("readonly.txt");
        fs::write(&path, b"test").unwrap();

        use windows::Win32::Storage::FileSystem::{SetFileAttributesW, FILE_ATTRIBUTE_READONLY};

        let wide_path: Vec<u16> = path.to_string_lossy().encode_utf16().collect();
        let _ = unsafe {
            SetFileAttributesW(
                windows::core::PCWSTR::from_raw(wide_path.as_ptr()),
                FILE_ATTRIBUTE_READONLY,
            )
        };

        let sddl = "D:P(A;;GA;;;SY)(A;;GA;;;BA)";
        let result = set_get_file_sddl(&path, sddl);

        assert!(result.is_err());
    }

    #[test]
    fn test_set_get_file_sddl_different_sddl() {
        let temp_dir = tempfile::tempdir().unwrap();
        let path = temp_dir.path().join("testfile.txt");
        fs::write(&path, b"test").unwrap();

        let sddl = "D:P(A;;GA;;;WD)(A;;GA;;;BA)";
        let result = set_get_file_sddl(&path, sddl).unwrap();

        assert_eq!(result, sddl);
    }
}
