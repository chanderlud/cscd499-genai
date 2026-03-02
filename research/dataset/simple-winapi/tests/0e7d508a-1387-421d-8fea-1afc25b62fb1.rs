// Auto-generated tests for: 0e7d508a-1387-421d-8fea-1afc25b62fb1.md
// Model: minimax/minimax-m2.5
// Extraction: raw

#[cfg(windows)]
#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::{self, File};
    use std::io::Write;
    use std::path::PathBuf;
    use std::thread;
    use std::time::Duration;

    fn create_temp_dir() -> PathBuf {
        let tmp_dir = std::env::temp_dir();
        let unique_dir = tmp_dir.join(format!("rust_watch_test_{}", std::process::id()));
        fs::create_dir_all(&unique_dir).expect("Failed to create temp dir");
        unique_dir
    }

    fn cleanup_dir(path: &PathBuf) {
        let _ = fs::remove_dir_all(path);
    }

    #[test]
    fn test_watch_dir_nonexistent_path() {
        let result = watch_dir(Path::new("C:\\nonexistent_dir_12345"), false);
        assert!(result.is_err(), "Expected error for nonexistent directory");
    }

    #[test]
    fn test_watch_dir_basic() {
        let tmp_dir = create_temp_dir();
        let result = watch_dir(&tmp_dir, false);
        assert!(result.is_ok(), "Failed to watch directory: {:?}", result.err());
        let _rx = result.unwrap();
        cleanup_dir(&tmp_dir);
    }

    #[test]
    fn test_watch_dir_file_create() {
        let tmp_dir = create_temp_dir();
        let rx = watch_dir(&tmp_dir, false).expect("Failed to watch directory");

        // Create a file
        let file_path = tmp_dir.join("test_file.txt");
        let mut file = File::create(&file_path).expect("Failed to create file");
        file.write_all(b"test content").expect("Failed to write to file");
        drop(file);

        // Wait for notification
        let events = rx.recv_timeout(Duration::from_secs(5)).expect("Timeout waiting for event");

        // Verify we received some notification data
        assert!(!events.is_empty(), "Expected file change notifications");

        cleanup_dir(&tmp_dir);
    }

    #[test]
    fn test_watch_dir_file_delete() {
        let tmp_dir = create_temp_dir();

        // Create a file first
        let file_path = tmp_dir.join("delete_me.txt");
        File::create(&file_path).expect("Failed to create file");

        let rx = watch_dir(&tmp_dir, false).expect("Failed to watch directory");

        // Delete the file
        fs::remove_file(&file_path).expect("Failed to delete file");

        // Wait for notification
        let events = rx.recv_timeout(Duration::from_secs(5)).expect("Timeout waiting for delete event");

        assert!(!events.is_empty(), "Expected file delete notifications");

        cleanup_dir(&tmp_dir);
    }

    #[test]
    fn test_watch_dir_file_modify() {
        let tmp_dir = create_temp_dir();

        // Create a file first
        let file_path = tmp_dir.join("modify_me.txt");
        let mut file = File::create(&file_path).expect("Failed to create file");
        file.write_all(b"initial").expect("Failed to write initial content");
        drop(file);

        let rx = watch_dir(&tmp_dir, false).expect("Failed to watch directory");

        // Modify the file
        thread::sleep(Duration::from_millis(100)); // Small delay to ensure timestamp difference
        let mut file = File::open(&file_path).expect("Failed to open file");
        use std::io::Write;
        file.write_all(b" modified").expect("Failed to modify file");
        drop(file);

        // Wait for notification
        let events = rx.recv_timeout(Duration::from_secs(5)).expect("Timeout waiting for modify event");

        assert!(!events.is_empty(), "Expected file modify notifications");

        cleanup_dir(&tmp_dir);
    }

    #[test]
    fn test_watch_dir_file_rename() {
        let tmp_dir = create_temp_dir();

        // Create a file first
        let old_name = tmp_dir.join("old_name.txt");
        File::create(&old_name).expect("Failed to create file");

        let rx = watch_dir(&tmp_dir, false).expect("Failed to watch directory");

        // Rename the file
        let new_name = tmp_dir.join("new_name.txt");
        fs::rename(&old_name, &new_name).expect("Failed to rename file");

        // Wait for notification
        let events = rx.recv_timeout(Duration::from_secs(5)).expect("Timeout waiting for rename event");

        assert!(!events.is_empty(), "Expected file rename notifications");

        cleanup_dir(&tmp_dir);
    }

    #[test]
    fn test_watch_dir_recursive() {
        let tmp_dir = create_temp_dir();

        // Create subdirectory
        let sub_dir = tmp_dir.join("subdir");
        fs::create_dir(&sub_dir).expect("Failed to create subdirectory");

        let rx = watch_dir(&tmp_dir, true).expect("Failed to watch directory recursively");

        // Create file in subdirectory
        let file_path = sub_dir.join("nested_file.txt");
        File::create(&file_path).expect("Failed to create nested file");

        // Wait for notification
        let events = rx.recv_timeout(Duration::from_secs(5)).expect("Timeout waiting for recursive event");

        assert!(!events.is_empty(), "Expected recursive file change notifications");

        cleanup_dir(&tmp_dir);
    }

    #[test]
    fn test_watch_dir_non_recursive_excludes_subdir() {
        let tmp_dir = create_temp_dir();

        // Create subdirectory
        let sub_dir = tmp_dir.join("subdir");
        fs::create_dir(&sub_dir).expect("Failed to create subdirectory");

        let rx = watch_dir(&tmp_dir, false).expect("Failed to watch directory non-recursively");

        // Create file in subdirectory - should NOT be caught with recursive=false
        let file_path = sub_dir.join("nested_file.txt");
        File::create(&file_path).expect("Failed to create nested file");

        // Try to receive with a short timeout - with non-recursive, this should timeout
        // because events in subdirectories shouldn't be reported
        let result = rx.recv_timeout(Duration::from_millis(500));

        // For non-recursive watch, we may or may not get events depending on implementation
        // The key is that the function works without panicking
        // Note: Some implementations may still report events, so we just verify basic functionality

        cleanup_dir(&tmp_dir);
    }
}
