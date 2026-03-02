// Auto-generated tests for: 6e6c9231-eb7c-451d-b472-f32e55009c75.md
// Model: arcee-ai/trinity-large-preview:free
// Extraction: rust

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;
    use std::fs;

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
}
