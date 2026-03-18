#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::os::windows::fs::OpenOptionsExt;
    use std::path::PathBuf;
    use windows::Win32::Foundation::{
        ERROR_FILE_NOT_FOUND, ERROR_PATH_NOT_FOUND, ERROR_SHARING_VIOLATION,
        ERROR_UNABLE_TO_REMOVE_REPLACED,
    };

    fn test_paths() -> (tempfile::TempDir, PathBuf, PathBuf) {
        let dir = tempfile::tempdir().unwrap();
        let src = dir.path().join("src.txt");
        let dst = dir.path().join("dst.txt");
        (dir, src, dst)
    }

    #[test]
    fn test_replace_file_happy_path() {
        let (_dir, src, dst) = test_paths();

        fs::write(&src, "source content").unwrap();
        fs::write(&dst, "destination content").unwrap();

        replace_file(&src, &dst).unwrap();

        let dst_content = fs::read_to_string(&dst).unwrap();
        assert_eq!(dst_content, "source content");
        assert!(!src.exists(), "source path should be consumed by replace");
    }

    #[test]
    fn test_replace_file_dst_not_exist() {
        let (_dir, src, dst) = test_paths();

        fs::write(&src, "source content").unwrap();

        replace_file(&src, &dst).unwrap();

        let dst_content = fs::read_to_string(&dst).unwrap();
        assert_eq!(dst_content, "source content");
        assert!(!src.exists(), "source path should be moved to destination");
    }

    #[test]
    fn test_replace_file_empty_files() {
        let (_dir, src, dst) = test_paths();

        fs::File::create(&src).unwrap();
        fs::File::create(&dst).unwrap();

        replace_file(&src, &dst).unwrap();

        let dst_size = fs::metadata(&dst).unwrap().len();
        assert_eq!(dst_size, 0);
        assert!(!src.exists(), "source path should be consumed by replace");
    }

    #[test]
    fn test_replace_file_same_file() {
        let (_dir, path_src, _) = test_paths();
        let path = path_src;

        fs::write(&path, "content").unwrap();

        replace_file(&path, &path).unwrap();

        let content = fs::read_to_string(&path).unwrap();
        assert_eq!(content, "content");
    }

    #[test]
    fn test_replace_file_src_not_exist() {
        let (_dir, src, dst) = test_paths();

        fs::write(&dst, "destination content").unwrap();

        let err = replace_file(&src, &dst).unwrap_err();
        let code = super::win32_code(&err);

        assert!(
            code == ERROR_FILE_NOT_FOUND.0 || code == ERROR_PATH_NOT_FOUND.0,
            "unexpected error: {err:?}"
        );

        let dst_content = fs::read_to_string(&dst).unwrap();
        assert_eq!(dst_content, "destination content");
    }

    #[test]
    fn test_replace_file_dst_locked() {
        let (_dir, src, dst) = test_paths();

        fs::write(&src, "source content").unwrap();
        fs::write(&dst, "destination content").unwrap();

        let err = {
            let _lock = fs::OpenOptions::new()
                .read(true)
                .write(true)
                .share_mode(0)
                .open(&dst)
                .unwrap();

            replace_file(&src, &dst).unwrap_err()
        }; // lock drops here

        let code = super::win32_code(&err);

        assert!(
            code == ERROR_SHARING_VIOLATION.0 || code == ERROR_UNABLE_TO_REMOVE_REPLACED.0,
            "unexpected error: {err:?}"
        );

        assert_eq!(fs::read_to_string(&src).unwrap(), "source content");
        assert_eq!(fs::read_to_string(&dst).unwrap(), "destination content");
    }
}
