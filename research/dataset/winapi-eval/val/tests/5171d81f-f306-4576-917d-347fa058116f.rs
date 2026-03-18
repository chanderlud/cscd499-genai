#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;
    use std::io::{Read, Write};
    use std::path::Path;
    use std::sync::atomic::AtomicBool;
    use std::sync::atomic::Ordering::Relaxed;
    use std::sync::Arc;
    use std::{fs, io};

    #[test]
    fn test_copy_with_progress_basic() {
        let src_path = Path::new("test_src.bin");
        let dst_path = Path::new("test_dst.bin");

        // Create source file with known content
        let content = b"Hello, world! This is a test file.";
        {
            let mut f = File::create(src_path).unwrap();
            f.write_all(content).unwrap();
        }

        let progress_called = Arc::new(AtomicBool::new(false));
        let progress_called_clone = progress_called.clone();
        let result = copy_with_progress(src_path, dst_path, |done, total| {
            progress_called_clone.store(true, Relaxed);
            // Only assert final progress values to allow intermediate callbacks
            if done == total {
                assert_eq!(done, content.len() as u64);
                assert_eq!(total, content.len() as u64);
            }
            true
        });

        assert!(result.is_ok());
        assert!(progress_called.load(Relaxed));

        // Verify file content
        let mut dst_content = Vec::new();
        let mut f = File::open(dst_path).unwrap();
        f.read_to_end(&mut dst_content).unwrap();
        assert_eq!(dst_content, content);

        // Cleanup
        fs::remove_file(src_path).unwrap();
        fs::remove_file(dst_path).unwrap();
    }

    #[test]
    fn test_copy_with_progress_cancel() {
        let src_path = Path::new("test_src_cancel.bin");
        let dst_path = Path::new("test_dst_cancel.bin");

        // Create source file
        let content = b"Small file";
        {
            let mut f = File::create(src_path).unwrap();
            f.write_all(content).unwrap();
        }

        // Cancel on first callback
        let result = copy_with_progress(src_path, dst_path, |_, _| false);

        assert!(result.is_err());

        // Cleanup
        fs::remove_file(src_path).unwrap();
        if dst_path.exists() {
            fs::remove_file(dst_path).unwrap();
        }
    }

    #[test]
    fn test_copy_with_progress_empty_file() {
        let src_path = Path::new("test_src_empty.bin");
        let dst_path = Path::new("test_dst_empty.bin");

        // Create empty source file
        File::create(src_path).unwrap();

        let progress_called = Arc::new(AtomicBool::new(false));
        let progress_called_clone = progress_called.clone();
        let result = copy_with_progress(src_path, dst_path, |done, total| {
            progress_called_clone.store(true, Relaxed);
            // For empty file, done and total should be 0
            assert_eq!(done, 0);
            assert_eq!(total, 0);
            true
        });

        assert!(result.is_ok());
        assert!(progress_called.load(Relaxed));

        // Verify destination is also empty
        let meta = fs::metadata(dst_path).unwrap();
        assert_eq!(meta.len(), 0);

        // Cleanup
        fs::remove_file(src_path).unwrap();
        fs::remove_file(dst_path).unwrap();
    }

    #[test]
    fn test_copy_with_progress_overwrite() {
        let src_path = Path::new("test_src_overwrite.bin");
        let dst_path = Path::new("test_dst_overwrite.bin");

        // Create source file
        let content = b"Source content";
        {
            let mut f = File::create(src_path).unwrap();
            f.write_all(content).unwrap();
        }

        // Create destination file with different content
        let old_content = b"Old content";
        {
            let mut f = File::create(dst_path).unwrap();
            f.write_all(old_content).unwrap();
        }

        let result = copy_with_progress(src_path, dst_path, |_, _| true);

        assert!(result.is_ok());

        // Verify destination now has source content
        let mut dst_content = Vec::new();
        let mut f = File::open(dst_path).unwrap();
        f.read_to_end(&mut dst_content).unwrap();
        assert_eq!(dst_content, content);

        // Cleanup
        fs::remove_file(src_path).unwrap();
        fs::remove_file(dst_path).unwrap();
    }
}
