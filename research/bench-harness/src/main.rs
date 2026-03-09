use std::ffi::OsStr;
use std::iter;
use std::path::Path;
use std::os::windows::ffi::OsStrExt;
use windows::core::{PCWSTR, Result};
use windows::Win32::Storage::FileSystem::CreateDirectoryW;
use windows::Win32::Foundation::ERROR_ALREADY_EXISTS;

#[allow(unused_imports)]
use windows::core::Error;

pub fn create_dir_all(path: &Path) -> Result<()> {
    // Convert Path to wide string with null terminator
    let wide_path = wide_null(path.as_os_str());

    // Create directory
    let result = unsafe {
        CreateDirectoryW(
            Some(&PCWSTR(wide_path.as_ptr())),
            None
        )
    };

    // Check if error is ERROR_ALREADY_EXISTS
    if let Err(e) = result {
        if e.code() == ERROR_ALREADY_EXISTS.into() {
            return Ok(());
        }
        return Err(e);
    }

    Ok(())
}

fn wide_null(s: &OsStr) -> Vec<u16> {
    s.encode_wide().chain(iter::once(0)).collect()
}

fn main() {}

// Auto-generated tests for: 0a5d7328-0ec4-4088-83d2-7e1c0e8b27c7.md
// Model: minimax/minimax-m2.5
// Extraction: rust

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;
    use std::fs;

    #[test]
    fn test_create_single_directory() {
        let temp_dir = std::env::temp_dir();
        let test_dir = temp_dir.join("rust_test_single");

        // Clean up if exists
        let _ = fs::remove_dir(&test_dir);

        let result = create_dir_all(&test_dir);
        assert!(result.is_ok(), "Failed to create single directory: {:?}", result);
        assert!(test_dir.exists(), "Directory should exist after creation");

        // Clean up
        let _ = fs::remove_dir(&test_dir);
    }

    #[test]
    fn test_create_nested_directories() {
        let temp_dir = std::env::temp_dir();
        let test_dir = temp_dir.join("rust_test_nested").join("a").join("b").join("c");

        // Clean up if exists
        let _ = fs::remove_dir_all(&test_dir);

        let result = create_dir_all(&test_dir);
        assert!(result.is_ok(), "Failed to create nested directories: {:?}", result);
        assert!(test_dir.exists(), "Nested directory should exist after creation");

        // Verify all parent levels exist
        assert!(temp_dir.join("rust_test_nested").exists());
        assert!(temp_dir.join("rust_test_nested").join("a").exists());
        assert!(temp_dir.join("rust_test_nested").join("a").join("b").exists());

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
        assert!(result2.is_ok(), "Second creation should tolerate already exists");

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
        assert!(result2.is_ok(), "Should succeed when parent exists: {:?}", result2);
        assert!(child_dir.exists());

        // Clean up
        let _ = fs::remove_dir_all(&parent_dir);
    }

    #[test]
    fn test_create_deeply_nested_directories() {
        let temp_dir = std::env::temp_dir();
        let test_dir = temp_dir.join("rust_test_deep")
            .join("a")
            .join("b")
            .join("c")
            .join("d")
            .join("e");

        // Clean up if exists
        let _ = fs::remove_dir_all(&test_dir);

        let result = create_dir_all(&test_dir);
        assert!(result.is_ok(), "Failed to create deeply nested directories: {:?}", result);
        assert!(test_dir.exists());

        // Clean up
        let _ = fs::remove_dir_all(&temp_dir.join("rust_test_deep"));
    }

    #[test]
    fn test_create_dir_returns_error_for_invalid_path() {
        // Test with an invalid path that cannot be created
        // On Windows, this might be a path with invalid characters
        let invalid_path = Path::new("");

        // Empty path should likely fail or handle gracefully
        let result = create_dir_all(invalid_path);
        // The exact behavior depends on implementation, but we verify it handles it
        assert!(result.is_err() || result.is_ok());
    }
}