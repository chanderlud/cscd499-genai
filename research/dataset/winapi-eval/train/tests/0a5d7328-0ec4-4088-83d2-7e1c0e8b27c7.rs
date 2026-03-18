#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::path::Path;

    #[test]
    fn test_create_single_directory() {
        let temp_dir = std::env::temp_dir();
        let test_dir = temp_dir.join("rust_test_single");

        // Clean up if exists
        let _ = fs::remove_dir(&test_dir);

        let result = create_dir_all(&test_dir);
        assert!(
            result.is_ok(),
            "Failed to create single directory: {:?}",
            result
        );
        assert!(test_dir.exists(), "Directory should exist after creation");

        // Clean up
        let _ = fs::remove_dir(&test_dir);
    }

    #[test]
    fn test_create_nested_directories() {
        let temp_dir = std::env::temp_dir();
        let test_dir = temp_dir
            .join("rust_test_nested")
            .join("a")
            .join("b")
            .join("c");

        // Clean up if exists
        let _ = fs::remove_dir_all(&test_dir);

        let result = create_dir_all(&test_dir);
        assert!(
            result.is_ok(),
            "Failed to create nested directories: {:?}",
            result
        );
        assert!(
            test_dir.exists(),
            "Nested directory should exist after creation"
        );

        // Verify all parent levels exist
        assert!(temp_dir.join("rust_test_nested").exists());
        assert!(temp_dir.join("rust_test_nested").join("a").exists());
        assert!(temp_dir
            .join("rust_test_nested")
            .join("a")
            .join("b")
            .exists());

        // Clean up
        let _ = fs::remove_dir_all(&temp_dir.join("rust_test_nested"));
    }

    #[test]
    fn test_create_dir_already_exists() {
        let temp_dir = std::env::temp_dir();
        let test_dir = temp_dir.join("rust_test_exists");

        // Clean up if exists
        let _ = fs::remove_dir(&test_dir);

        // First creation should succeed
        let result1 = create_dir_all(&test_dir);
        assert!(result1.is_ok(), "First creation should succeed");
        assert!(test_dir.exists());

        // Second creation should also succeed (tolerate "already exists")
        let result2 = create_dir_all(&test_dir);
        assert!(
            result2.is_ok(),
            "Second creation should tolerate already exists"
        );

        // Clean up
        let _ = fs::remove_dir(&test_dir);
    }

    #[test]
    fn test_create_dir_with_existing_parent() {
        let temp_dir = std::env::temp_dir();
        let parent_dir = temp_dir.join("rust_test_parent");
        let child_dir = parent_dir.join("child");

        // Clean up if exists
        let _ = fs::remove_dir_all(&parent_dir);

        // Create parent first
        let result1 = create_dir_all(&parent_dir);
        assert!(result1.is_ok());

        // Create child with existing parent
        let result2 = create_dir_all(&child_dir);
        assert!(
            result2.is_ok(),
            "Should succeed when parent exists: {:?}",
            result2
        );
        assert!(child_dir.exists());

        // Clean up
        let _ = fs::remove_dir_all(&parent_dir);
    }

    #[test]
    fn test_create_deeply_nested_directories() {
        let temp_dir = std::env::temp_dir();
        let test_dir = temp_dir
            .join("rust_test_deep")
            .join("a")
            .join("b")
            .join("c")
            .join("d")
            .join("e");

        // Clean up if exists
        let _ = fs::remove_dir_all(&test_dir);

        let result = create_dir_all(&test_dir);
        assert!(
            result.is_ok(),
            "Failed to create deeply nested directories: {:?}",
            result
        );
        assert!(test_dir.exists());

        // Clean up
        let _ = fs::remove_dir_all(&temp_dir.join("rust_test_deep"));
    }

    #[test]
    fn test_create_dir_returns_error_for_invalid_path() {
        // Test with an invalid path that cannot be created
        // On Windows, this might be a path with invalid characters
        let invalid_path = Path::new("bad*path");

        // Empty path should likely fail or handle gracefully
        let result = create_dir_all(invalid_path);
        assert!(result.is_err());
    }

    #[test]
    fn test_create_dir_with_special_characters_in_path() {
        let temp_dir = std::env::temp_dir();
        let test_dir = temp_dir.join("rust_test_special").join("test!@#$%^&()_+");

        // Clean up if exists
        let _ = fs::remove_dir_all(&test_dir);

        let result = create_dir_all(&test_dir);
        assert!(
            result.is_ok(),
            "Should handle special characters in path: {:?}",
            result
        );
        assert!(test_dir.exists());

        // Clean up
        let _ = fs::remove_dir_all(&temp_dir.join("rust_test_special"));
    }

    #[test]
    fn test_create_dir_with_long_path() {
        let temp_dir = std::env::temp_dir();
        let mut test_dir = temp_dir.join("rust_test_long");
        for i in 0..10 {
            test_dir = test_dir.join(format!("dir_{}", i));
        }

        // Clean up if exists
        let _ = fs::remove_dir_all(&test_dir);

        let result = create_dir_all(&test_dir);
        assert!(result.is_ok(), "Should handle long paths: {:?}", result);
        assert!(test_dir.exists());

        // Clean up
        let _ = fs::remove_dir_all(&temp_dir.join("rust_test_long"));
    }

    #[test]
    fn test_create_dir_with_existing_intermediate_directories() {
        let temp_dir = std::env::temp_dir();
        let test_dir = temp_dir
            .join("rust_test_partial")
            .join("a")
            .join("b")
            .join("c");

        // Clean up if exists
        let _ = fs::remove_dir_all(&test_dir);

        // Create partial structure
        let partial_dir = temp_dir.join("rust_test_partial").join("a");
        let _ = fs::create_dir_all(&partial_dir);

        let result = create_dir_all(&test_dir);
        assert!(
            result.is_ok(),
            "Should succeed with existing intermediate directories: {:?}",
            result
        );
        assert!(test_dir.exists());

        // Clean up
        let _ = fs::remove_dir_all(&temp_dir.join("rust_test_partial"));
    }
}
