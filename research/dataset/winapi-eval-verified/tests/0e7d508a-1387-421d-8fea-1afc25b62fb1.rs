#[cfg(windows)]
#[cfg(test)]
mod tests {
    use super::*;
    use std::convert::TryInto;
    use std::fs::{self, File, OpenOptions};
    use std::io::Write;
    use std::path::PathBuf;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::sync::mpsc::{Receiver, RecvTimeoutError};
    use std::thread;
    use std::time::{Duration, Instant};

    static COUNTER: AtomicUsize = AtomicUsize::new(0);

    const FILE_ACTION_ADDED: u32 = 0x00000001;
    const FILE_ACTION_REMOVED: u32 = 0x00000002;
    const FILE_ACTION_MODIFIED: u32 = 0x00000003;
    const FILE_ACTION_RENAMED_OLD_NAME: u32 = 0x00000004;
    const FILE_ACTION_RENAMED_NEW_NAME: u32 = 0x00000005;

    #[derive(Debug, Clone)]
    struct NotifyRecord {
        action: u32,
        name: String,
    }

    fn create_temp_dir() -> PathBuf {
        let tmp_dir = std::env::temp_dir();
        let unique_dir = tmp_dir.join(format!(
            "rust_watch_test_{}_{}",
            std::process::id(),
            COUNTER.fetch_add(1, Ordering::Relaxed)
        ));
        let _ = fs::remove_dir_all(&unique_dir);
        fs::create_dir_all(&unique_dir).expect("Failed to create temp dir");
        unique_dir
    }

    fn cleanup_dir(path: &PathBuf) {
        let _ = fs::remove_dir_all(path);
    }

    fn parse_notify_records(buf: &[u8]) -> Vec<NotifyRecord> {
        let mut out = Vec::new();
        let mut offset = 0usize;

        while offset + 12 <= buf.len() {
            let next_entry_offset =
                u32::from_le_bytes(buf[offset..offset + 4].try_into().unwrap()) as usize;
            let action =
                u32::from_le_bytes(buf[offset + 4..offset + 8].try_into().unwrap());
            let file_name_len =
                u32::from_le_bytes(buf[offset + 8..offset + 12].try_into().unwrap()) as usize;

            let name_start = offset + 12;
            let name_end = name_start.saturating_add(file_name_len);

            if name_end > buf.len() || file_name_len % 2 != 0 {
                break;
            }

            let utf16_name: Vec<u16> = buf[name_start..name_end]
                .chunks_exact(2)
                .map(|c| u16::from_le_bytes([c[0], c[1]]))
                .collect();

            out.push(NotifyRecord {
                action,
                name: String::from_utf16_lossy(&utf16_name),
            });

            if next_entry_offset == 0 {
                break;
            }

            offset = offset.saturating_add(next_entry_offset);
        }

        out
    }

    fn recv_until<F>(rx: &Receiver<Vec<u8>>, timeout: Duration, mut predicate: F) -> bool
    where
        F: FnMut(&[NotifyRecord]) -> bool,
    {
        let deadline = Instant::now() + timeout;

        loop {
            let now = Instant::now();
            if now >= deadline {
                return false;
            }

            match rx.recv_timeout(deadline - now) {
                Ok(buf) => {
                    let records = parse_notify_records(&buf);
                    if predicate(&records) {
                        return true;
                    }
                }
                Err(RecvTimeoutError::Timeout) => return false,
                Err(RecvTimeoutError::Disconnected) => return false,
            }
        }
    }

    fn arm_watch() {
        thread::sleep(Duration::from_millis(100));
    }

    #[test]
    fn test_watch_dir_nonexistent_path() {
        let result = watch_dir(Path::new("C:\\nonexistent_dir_12345"), false);
        assert!(result.is_err(), "Expected error for nonexistent directory");
    }

    #[test]
    fn test_watch_dir_path_is_file() {
        let tmp_dir = create_temp_dir();
        let file_path = tmp_dir.join("not_a_directory.txt");
        File::create(&file_path).expect("Failed to create file");

        let result = watch_dir(&file_path, false);
        assert!(result.is_err(), "Expected error when path is a file");

        cleanup_dir(&tmp_dir);
    }

    #[test]
    fn test_watch_dir_basic() {
        let tmp_dir = create_temp_dir();
        let result = watch_dir(&tmp_dir, false);
        assert!(result.is_ok(), "Failed to watch directory: {:?}", result.err());

        let rx = result.unwrap();
        drop(rx);
        cleanup_dir(&tmp_dir);
    }

    #[test]
    fn test_watch_dir_file_create() {
        let tmp_dir = create_temp_dir();
        let rx = watch_dir(&tmp_dir, false).expect("Failed to watch directory");
        arm_watch();

        let file_path = tmp_dir.join("test_file.txt");
        let mut file = File::create(&file_path).expect("Failed to create file");
        file.write_all(b"test content").expect("Failed to write to file");
        file.sync_all().expect("Failed to sync file");
        drop(file);

        let saw_create = recv_until(&rx, Duration::from_secs(5), |records| {
            records.iter().any(|r| r.action == FILE_ACTION_ADDED)
        });

        assert!(saw_create, "Expected a FILE_ACTION_ADDED notification");

        drop(rx);
        cleanup_dir(&tmp_dir);
    }

    #[test]
    fn test_watch_dir_file_delete() {
        let tmp_dir = create_temp_dir();

        let file_path = tmp_dir.join("delete_me.txt");
        File::create(&file_path).expect("Failed to create file");

        let rx = watch_dir(&tmp_dir, false).expect("Failed to watch directory");
        arm_watch();

        fs::remove_file(&file_path).expect("Failed to delete file");

        let saw_delete = recv_until(&rx, Duration::from_secs(5), |records| {
            records.iter().any(|r| r.action == FILE_ACTION_REMOVED)
        });

        assert!(saw_delete, "Expected a FILE_ACTION_REMOVED notification");

        drop(rx);
        cleanup_dir(&tmp_dir);
    }

    #[test]
    fn test_watch_dir_file_modify() {
        let tmp_dir = create_temp_dir();

        let file_path = tmp_dir.join("modify_me.txt");
        let mut file = File::create(&file_path).expect("Failed to create file");
        file.write_all(b"initial").expect("Failed to write initial content");
        file.sync_all().expect("Failed to sync initial content");
        drop(file);

        let rx = watch_dir(&tmp_dir, false).expect("Failed to watch directory");
        arm_watch();

        thread::sleep(Duration::from_millis(100));
        let mut file = OpenOptions::new()
            .append(true)
            .open(&file_path)
            .expect("Failed to open file for modification");
        file.write_all(b" modified").expect("Failed to modify file");
        file.sync_all().expect("Failed to sync modified file");
        drop(file);

        let saw_modify = recv_until(&rx, Duration::from_secs(5), |records| {
            records.iter().any(|r| r.action == FILE_ACTION_MODIFIED)
        });

        assert!(saw_modify, "Expected a FILE_ACTION_MODIFIED notification");

        drop(rx);
        cleanup_dir(&tmp_dir);
    }

    #[test]
    fn test_watch_dir_file_rename() {
        let tmp_dir = create_temp_dir();

        let old_name = tmp_dir.join("old_name.txt");
        File::create(&old_name).expect("Failed to create file");

        let rx = watch_dir(&tmp_dir, false).expect("Failed to watch directory");
        arm_watch();

        let new_name = tmp_dir.join("new_name.txt");
        fs::rename(&old_name, &new_name).expect("Failed to rename file");

        let deadline = Instant::now() + Duration::from_secs(5);
        let mut saw_old = false;
        let mut saw_new = false;

        while Instant::now() < deadline && !(saw_old && saw_new) {
            match rx.recv_timeout(deadline - Instant::now()) {
                Ok(buf) => {
                    for record in parse_notify_records(&buf) {
                        if record.action == FILE_ACTION_RENAMED_OLD_NAME {
                            saw_old = true;
                        }
                        if record.action == FILE_ACTION_RENAMED_NEW_NAME {
                            saw_new = true;
                        }
                    }
                }
                Err(RecvTimeoutError::Timeout) => break,
                Err(RecvTimeoutError::Disconnected) => break,
            }
        }

        assert!(saw_old, "Expected FILE_ACTION_RENAMED_OLD_NAME notification");
        assert!(saw_new, "Expected FILE_ACTION_RENAMED_NEW_NAME notification");

        drop(rx);
        cleanup_dir(&tmp_dir);
    }

    #[test]
    fn test_watch_dir_recursive() {
        let tmp_dir = create_temp_dir();

        let sub_dir = tmp_dir.join("subdir");
        fs::create_dir(&sub_dir).expect("Failed to create subdirectory");

        let rx = watch_dir(&tmp_dir, true).expect("Failed to watch directory recursively");
        arm_watch();

        let file_path = sub_dir.join("nested_file.txt");
        let mut file = File::create(&file_path).expect("Failed to create nested file");
        file.write_all(b"nested").expect("Failed to write nested file");
        file.sync_all().expect("Failed to sync nested file");
        drop(file);

        let saw_nested_relative_path = recv_until(&rx, Duration::from_secs(5), |records| {
            records.iter().any(|r| r.name.contains('\\') || r.name.contains('/'))
        });

        assert!(
            saw_nested_relative_path,
            "Expected a recursive notification containing a nested relative path"
        );

        drop(rx);
        cleanup_dir(&tmp_dir);
    }

    #[test]
    fn test_watch_dir_non_recursive_excludes_nested_relative_paths() {
        let tmp_dir = create_temp_dir();

        let sub_dir = tmp_dir.join("subdir");
        fs::create_dir(&sub_dir).expect("Failed to create subdirectory");

        let rx = watch_dir(&tmp_dir, false).expect("Failed to watch directory non-recursively");
        arm_watch();

        let file_path = sub_dir.join("nested_file.txt");
        let mut file = File::create(&file_path).expect("Failed to create nested file");
        file.write_all(b"nested").expect("Failed to write nested file");
        file.sync_all().expect("Failed to sync nested file");
        drop(file);

        let saw_nested_relative_path = recv_until(&rx, Duration::from_secs(2), |records| {
            records.iter().any(|r| r.name.contains('\\') || r.name.contains('/'))
        });

        assert!(
            !saw_nested_relative_path,
            "Did not expect a non-recursive watch to report nested relative paths"
        );

        drop(rx);
        cleanup_dir(&tmp_dir);
    }
}