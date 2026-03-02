#[cfg(test)]
mod tests {
    #[cfg(windows)]
    use super::create_hardlink_and_verify;

    #[cfg(windows)]
    use std::{
        fs,
        io::{self, Write},
        path::{Path, PathBuf},
        time::{SystemTime, UNIX_EPOCH},
    };

    #[cfg(windows)]
    fn unique_temp_dir(test_name: &str) -> PathBuf {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let mut dir = std::env::temp_dir();
        dir.push(format!(
            "hardlink_verify_{}_{}_{}",
            test_name,
            std::process::id(),
            nanos
        ));
        fs::create_dir_all(&dir).unwrap();
        dir
    }

    /// Some environments (non-NTFS volumes, weird sandboxes) may not support hard links.
    /// If so, these tests should skip rather than fail for the wrong reason.
    #[cfg(windows)]
    fn is_hardlink_not_supported(err: &io::Error) -> bool {
        matches!(err.kind(), io::ErrorKind::Unsupported)
            || matches!(err.raw_os_error(), Some(1) | Some(50)) // ERROR_INVALID_FUNCTION (1), ERROR_NOT_SUPPORTED (50)
    }

    #[cfg(windows)]
    fn assert_already_exists(err: &io::Error) {
        let ok = matches!(err.kind(), io::ErrorKind::AlreadyExists)
            || matches!(err.raw_os_error(), Some(80) | Some(183)); // ERROR_FILE_EXISTS (80), ERROR_ALREADY_EXISTS (183)
        assert!(
            ok,
            "expected AlreadyExists-ish error, got kind={:?} raw_os_error={:?}",
            err.kind(),
            err.raw_os_error()
        );
    }

    #[cfg(windows)]
    fn assert_not_found(err: &io::Error) {
        assert!(
            matches!(err.kind(), io::ErrorKind::NotFound)
                || matches!(err.raw_os_error(), Some(2) | Some(3)), // ERROR_FILE_NOT_FOUND (2), ERROR_PATH_NOT_FOUND (3)
            "expected NotFound-ish error, got kind={:?} raw_os_error={:?}",
            err.kind(),
            err.raw_os_error()
        );
    }

    #[cfg(windows)]
    fn assert_permission_denied(err: &io::Error) {
        assert!(
            matches!(err.kind(), io::ErrorKind::PermissionDenied)
                || matches!(err.raw_os_error(), Some(5)), // ERROR_ACCESS_DENIED (5)
            "expected PermissionDenied-ish error, got kind={:?} raw_os_error={:?}",
            err.kind(),
            err.raw_os_error()
        );
    }

    #[cfg(windows)]
    fn best_effort_cleanup(dir: &Path) {
        // Best-effort cleanup. If something is holding a handle open, this often fails.
        let _ = fs::remove_dir_all(dir);
    }

    #[cfg(windows)]
    #[test]
    fn creates_hardlink_and_verifies_identity_and_content() {
        let dir = unique_temp_dir("smoke");
        let existing = dir.join("orig.txt");
        let new_link = dir.join("orig_link.txt");

        fs::write(&existing, b"hello").unwrap();

        let ok = match create_hardlink_and_verify(&existing, &new_link) {
            Ok(v) => v,
            Err(e) if is_hardlink_not_supported(&e) => {
                best_effort_cleanup(&dir);
                return; // skip
            }
            Err(e) => panic!("unexpected error: {e:?}"),
        };
        assert!(ok, "expected identity verification to succeed");
        assert!(new_link.exists(), "new_link should exist after creation");

        // Content should be identical via either path.
        assert_eq!(fs::read(&new_link).unwrap(), b"hello");

        // Mutate through one name; observe via the other (same underlying file).
        {
            let mut f = fs::OpenOptions::new()
                .append(true)
                .open(&existing)
                .unwrap();
            f.write_all(b" world").unwrap();
            f.flush().unwrap();
        }
        assert_eq!(fs::read(&new_link).unwrap(), b"hello world");

        // Handles should be closed by the function: cleanup should succeed.
        fs::remove_file(&new_link).unwrap();
        fs::remove_file(&existing).unwrap();
        best_effort_cleanup(&dir);
    }

    #[cfg(windows)]
    #[test]
    fn errors_when_existing_missing() {
        let dir = unique_temp_dir("missing_existing");
        let existing = dir.join("does_not_exist.txt");
        let new_link = dir.join("link.txt");

        match create_hardlink_and_verify(&existing, &new_link) {
            Ok(v) => panic!("expected error, got Ok({v})"),
            Err(e) if is_hardlink_not_supported(&e) => {
                best_effort_cleanup(&dir);
                return; // skip
            }
            Err(e) => assert_not_found(&e),
        }

        best_effort_cleanup(&dir);
    }

    #[cfg(windows)]
    #[test]
    fn errors_when_new_link_already_exists() {
        let dir = unique_temp_dir("dest_exists");
        let existing = dir.join("orig.txt");
        let new_link = dir.join("orig_link.txt");

        fs::write(&existing, b"hello").unwrap();
        fs::write(&new_link, b"i already exist").unwrap();

        match create_hardlink_and_verify(&existing, &new_link) {
            Ok(v) => panic!("expected error, got Ok({v})"),
            Err(e) if is_hardlink_not_supported(&e) => {
                best_effort_cleanup(&dir);
                return; // skip
            }
            Err(e) => assert_already_exists(&e),
        }

        // Cleanup
        let _ = fs::remove_file(&new_link);
        let _ = fs::remove_file(&existing);
        best_effort_cleanup(&dir);
    }

    #[cfg(windows)]
    #[test]
    fn errors_when_existing_is_directory() {
        let dir = unique_temp_dir("existing_is_dir");
        let existing_dir = dir.join("orig_dir");
        let new_link = dir.join("dir_link");

        fs::create_dir_all(&existing_dir).unwrap();

        match create_hardlink_and_verify(&existing_dir, &new_link) {
            Ok(v) => panic!("expected error, got Ok({v})"),
            Err(e) if is_hardlink_not_supported(&e) => {
                best_effort_cleanup(&dir);
                return; // skip
            }
            Err(e) => {
                // Commonly PermissionDenied / ERROR_ACCESS_DENIED.
                assert_permission_denied(&e);
            }
        }

        best_effort_cleanup(&dir);
    }

    #[cfg(windows)]
    #[test]
    fn works_with_unicode_paths() {
        let dir = unique_temp_dir("unicode");
        let existing = dir.join("оригинал_文件.txt");
        let new_link = dir.join("ссылка_链接.txt");

        fs::write(&existing, "data").unwrap();

        let ok = match create_hardlink_and_verify(&existing, &new_link) {
            Ok(v) => v,
            Err(e) if is_hardlink_not_supported(&e) => {
                best_effort_cleanup(&dir);
                return; // skip
            }
            Err(e) => panic!("unexpected error: {e:?}"),
        };
        assert!(ok);
        assert_eq!(fs::read_to_string(&new_link).unwrap(), "data");

        fs::remove_file(&new_link).unwrap();
        fs::remove_file(&existing).unwrap();
        best_effort_cleanup(&dir);
    }
}