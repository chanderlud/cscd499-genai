#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::path::Path;

    #[test]
    fn test_overlapped_write_all_basic() {
        let path = Path::new("C:\\Temp\\ovl_basic.bin");
        let data = b"hello";
        let result = overlapped_write_all(path, data);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 5);
        assert!(path.exists());
        let written = fs::read(path).unwrap();
        assert_eq!(&written, data);
    }

    #[test]
    fn test_overlapped_write_all_empty_data() {
        let path = Path::new("C:\\Temp\\ovl_empty.bin");
        let data: &[u8] = b"";
        let result = overlapped_write_all(path, data);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 0);
        assert!(path.exists());
        let written = fs::read(path).unwrap();
        assert_eq!(&written, data);
    }

    #[test]
    fn test_overlapped_write_all_long_data() {
        let path = Path::new("C:\\Temp\\ovl_long.bin");
        let data = vec![42u8; 10_000];
        let result = overlapped_write_all(path, &data);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 10_000);
        assert!(path.exists());
        let written = fs::read(path).unwrap();
        assert_eq!(&written, &data);
    }

    #[test]
    fn test_overlapped_write_all_overwrite() {
        let path = Path::new("C:\\Temp\\ovl_overwrite.bin");
        let initial = b"old";
        fs::write(path, initial).unwrap();

        let data = b"new";
        let result = overlapped_write_all(path, data);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 3);
        assert!(path.exists());
        let written = fs::read(path).unwrap();
        assert_eq!(&written, data);
    }

    #[test]
    fn test_overlapped_write_all_invalid_path() {
        use std::time::{SystemTime, UNIX_EPOCH};

        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();

        // Create a path whose parent directory definitely does not exist.
        let missing_parent = std::env::temp_dir().join(format!(
            "ovl_missing_parent_{}_{}",
            std::process::id(),
            unique
        ));

        let path = missing_parent.join("ovl_invalid.bin");
        let data = b"test";

        assert!(!missing_parent.exists());

        let result = overlapped_write_all(&path, data);
        assert!(result.is_err());
    }
    #[test]
    fn test_overlapped_write_all_path_too_long() {
        let path = Path::new("C:\\Temp\\ovl_").join("a".repeat(300));
        let data = b"test";
        let result = overlapped_write_all(&path, data);
        assert!(result.is_err());
    }

    #[test]
    fn test_overlapped_write_all_no_permission() {
        let path = Path::new("C:\\Windows\\ovl_protected.bin");
        let data = b"test";
        let result = overlapped_write_all(path, data);
        assert!(result.is_err());
    }
}