#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::io::Write;
    use std::path::Path;
    use tempfile::NamedTempFile;

    #[test]
    fn test_copy_with_progress_small_file_success() {
        let src = NamedTempFile::new().unwrap();
        let dst = NamedTempFile::new().unwrap();

        // Write 100 bytes to source
        src.as_file().write_all(&[0u8; 100]).unwrap();

        let transferred = copy_with_progress(src.path(), dst.path(), None).unwrap();
        assert_eq!(transferred, 100);
        assert_eq!(fs::metadata(src.path()).unwrap().len(), 100);
        assert_eq!(fs::metadata(dst.path()).unwrap().len(), 100);
    }

    #[test]
    fn test_copy_with_progress_cancel_after_threshold() {
        let src = NamedTempFile::new().unwrap();
        let dst = NamedTempFile::new().unwrap();

        // Write 2KB to source
        src.as_file().write_all(&[0u8; 2048]).unwrap();

        // Cancel after 1KB
        let result = copy_with_progress(src.path(), dst.path(), Some(1024));
        assert!(result.is_err());
    }

    #[test]
    fn test_copy_with_progress_exact_threshold() {
        let src = NamedTempFile::new().unwrap();
        let dst = NamedTempFile::new().unwrap();

        // Write 1KB to source
        src.as_file().write_all(&[0u8; 1024]).unwrap();

        // Cancel after exactly 1KB
        let result = copy_with_progress(src.path(), dst.path(), Some(1024));
        assert!(result.is_err());
    }

    #[test]
    fn test_copy_with_progress_empty_file() {
        let src = NamedTempFile::new().unwrap();
        let dst = NamedTempFile::new().unwrap();

        let transferred = copy_with_progress(src.path(), dst.path(), None).unwrap();
        assert_eq!(transferred, 0);
        assert_eq!(fs::metadata(dst.path()).unwrap().len(), 0);
    }

    #[test]
    fn test_copy_with_progress_large_file_no_cancel() {
        let src = NamedTempFile::new().unwrap();
        let dst = NamedTempFile::new().unwrap();

        // Write 10MB to source
        src.as_file().write_all(&[0u8; 10 * 1024 * 1024]).unwrap();

        let transferred = copy_with_progress(src.path(), dst.path(), None).unwrap();
        assert_eq!(transferred, 10 * 1024 * 1024);
        assert_eq!(fs::metadata(dst.path()).unwrap().len(), 10 * 1024 * 1024);
    }

    #[test]
    fn test_copy_with_progress_cancel_after_zero() {
        let src = NamedTempFile::new().unwrap();
        let dst = NamedTempFile::new().unwrap();

        // Write 100 bytes to source
        src.as_file().write_all(&[0u8; 100]).unwrap();

        // Cancel after 0 bytes (should fail immediately)
        let result = copy_with_progress(src.path(), dst.path(), Some(0));
        assert!(result.is_err());
    }

    #[test]
    fn test_copy_with_progress_nonexistent_source() {
        let src = Path::new(r"C:\nonexistent\file.txt");
        let dst = NamedTempFile::new().unwrap();

        let result = copy_with_progress(src, dst.path(), None);
        assert!(result.is_err());
    }

    #[test]
    fn test_copy_with_progress_same_file() {
        let file = NamedTempFile::new().unwrap();

        let result = copy_with_progress(file.path(), file.path(), None);
        assert!(result.is_err());
    }
}
