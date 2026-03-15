#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::io::Write;

    #[test]
    fn test_replace_file_happy_path() {
        let src = std::env::temp_dir().join("src.txt");
        let dst = std::env::temp_dir().join("dst.txt");

        // Setup: create source and destination files
        fs::write(&src, "source content").unwrap();
        fs::write(&dst, "destination content").unwrap();

        // Act: replace destination with source
        replace_file(&src, &dst).unwrap();

        // Assert: destination now has source content
        let dst_content = fs::read_to_string(&dst).unwrap();
        assert_eq!(dst_content, "source content");

        // Cleanup
        fs::remove_file(&src).unwrap();
        fs::remove_file(&dst).unwrap();
    }

    #[test]
    fn test_replace_file_dst_not_exist() {
        let src = std::env::temp_dir().join("src.txt");
        let dst = std::env::temp_dir().join("dst.txt");

        // Setup: create source only
        fs::write(&src, "source content").unwrap();

        // Act: replace non-existent destination
        replace_file(&src, &dst).unwrap();

        // Assert: destination now exists with source content
        let dst_content = fs::read_to_string(&dst).unwrap();
        assert_eq!(dst_content, "source content");

        // Cleanup
        fs::remove_file(&src).unwrap();
        fs::remove_file(&dst).unwrap();
    }

    #[test]
    fn test_replace_file_empty_files() {
        let src = std::env::temp_dir().join("src.txt");
        let dst = std::env::temp_dir().join("dst.txt");

        // Setup: create empty source and destination
        fs::File::create(&src).unwrap();
        fs::File::create(&dst).unwrap();

        // Act
        replace_file(&src, &dst).unwrap();

        // Assert: both files are empty
        let src_size = fs::metadata(&src).unwrap().len();
        let dst_size = fs::metadata(&dst).unwrap().len();
        assert_eq!(src_size, 0);
        assert_eq!(dst_size, 0);

        // Cleanup
        fs::remove_file(&src).unwrap();
        fs::remove_file(&dst).unwrap();
    }

    #[test]
    fn test_replace_file_same_file() {
        let path = std::env::temp_dir().join("same.txt");

        // Setup: create file with content
        fs::write(&path, "content").unwrap();

        // Act: replace with itself
        replace_file(&path, &path).unwrap();

        // Assert: content unchanged
        let content = fs::read_to_string(&path).unwrap();
        assert_eq!(content, "content");

        // Cleanup
        fs::remove_file(&path).unwrap();
    }

    #[test]
    fn test_replace_file_src_not_exist() {
        let src = std::env::temp_dir().join("missing_src.txt");
        let dst = std::env::temp_dir().join("dst.txt");

        // Setup: create destination only
        fs::write(&dst, "destination content").unwrap();

        // Act: try to replace with non-existent source
        let err = replace_file(&src, &dst).unwrap_err();

        // Assert: should return an error
        assert!(err.to_string().contains("NotFound") || err.to_string().contains("failed"));

        // Cleanup
        fs::remove_file(&dst).unwrap();
    }

    #[test]
    fn test_replace_file_permissions_error() {
        let src = std::env::temp_dir().join("src.txt");
        let dst = std::env::temp_dir().join("dst.txt");

        // Setup: create source and destination
        fs::write(&src, "source content").unwrap();
        fs::write(&dst, "destination content").unwrap();

        // Make destination read-only
        let mut perms = fs::metadata(&dst).unwrap().permissions();
        perms.set_readonly(true);
        fs::set_permissions(&dst, perms).unwrap();

        // Act: attempt replace (should fail due to permissions)
        let err = replace_file(&src, &dst).unwrap_err();

        // Assert: should return an error
        assert!(err.to_string().contains("permission") || err.to_string().contains("failed"));

        // Cleanup: restore permissions and remove files
        let mut perms = fs::metadata(&dst).unwrap().permissions();
        perms.set_readonly(false);
        fs::set_permissions(&dst, perms).unwrap();
        fs::remove_file(&src).unwrap();
        fs::remove_file(&dst).unwrap();
    }
}
