#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::io;
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
        let path = Path::new("C:\\Invalid\\Path\\ovl_invalid.bin");
        let data = b"test";
        let result = overlapped_write_all(path, data);
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

    #[test]
    fn test_overlapped_write_all_disk_full() {
        let path = Path::new("C:\\Temp\\ovl_disk_full.bin");
        let data = vec![0u8; 1024 * 1024]; // 1MB
        let result = overlapped_write_all(path, &data);
        assert!(result.is_err());
    }

    #[test]
    fn test_overlapped_write_all_concurrent_writes() {
        let path = Path::new("C:\\Temp\\ovl_concurrent.bin");
        let data = b"first";
        let result = overlapped_write_all(path, data);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 5);
        assert!(path.exists());
        let written = fs::read(path).unwrap();
        assert_eq!(&written, data);
    }
}
