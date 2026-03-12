#[cfg(test)]
mod tests {
    use super::*;

    use std::fs;
    use std::os::windows::fs::MetadataExt;
    use std::path::{Path, PathBuf};
    use std::sync::atomic::{AtomicU64, Ordering};

    use windows::Win32::Foundation::FILETIME;

    // Fixed FILETIME values on whole-second boundaries.
    const FT_2020_01_01_UTC: u64 = 132_223_104_000_000_000;
    const FT_2021_01_01_UTC: u64 = 132_539_328_000_000_000;

    static NEXT_ID: AtomicU64 = AtomicU64::new(0);

    struct TempFile {
        path: PathBuf,
    }

    impl TempFile {
        fn new(test_name: &str) -> Self {
            let path = unique_path(test_name);
            let _ = fs::remove_file(&path);
            fs::write(&path, b"timestamp test payload").unwrap();
            Self { path }
        }

        fn path(&self) -> &Path {
            &self.path
        }
    }

    impl Drop for TempFile {
        fn drop(&mut self) {
            let _ = fs::remove_file(&self.path);
        }
    }

    fn unique_path(test_name: &str) -> PathBuf {
        let id = NEXT_ID.fetch_add(1, Ordering::Relaxed);
        std::env::temp_dir().join(format!(
            "set_last_write_time_{test_name}_{}_{}.tmp",
            std::process::id(),
            id
        ))
    }

    fn filetime_from_raw(raw: u64) -> FILETIME {
        FILETIME {
            dwLowDateTime: raw as u32,
            dwHighDateTime: (raw >> 32) as u32,
        }
    }

    fn timestamps(path: &Path) -> (u64, u64, u64) {
        let metadata = fs::metadata(path).unwrap();
        (
            metadata.creation_time(),
            metadata.last_access_time(),
            metadata.last_write_time(),
        )
    }

    #[test]
    fn sets_only_last_write_time() {
        let file = TempFile::new("sets_only_last_write_time");

        let (created_before, accessed_before, written_before) = timestamps(file.path());
        assert_ne!(written_before, FT_2020_01_01_UTC);

        set_last_write_time(file.path(), filetime_from_raw(FT_2020_01_01_UTC)).unwrap();

        let (created_after, accessed_after, written_after) = timestamps(file.path());

        assert_eq!(created_after, created_before);
        assert_eq!(accessed_after, accessed_before);
        assert_eq!(written_after, FT_2020_01_01_UTC);
    }

    #[test]
    fn can_overwrite_last_write_time_more_than_once() {
        let file = TempFile::new("can_overwrite_last_write_time_more_than_once");

        let (created_before, accessed_before, _) = timestamps(file.path());

        set_last_write_time(file.path(), filetime_from_raw(FT_2020_01_01_UTC)).unwrap();
        let (_, _, written_after_first_set) = timestamps(file.path());
        assert_eq!(written_after_first_set, FT_2020_01_01_UTC);

        set_last_write_time(file.path(), filetime_from_raw(FT_2021_01_01_UTC)).unwrap();

        let (created_after, accessed_after, written_after_second_set) = timestamps(file.path());

        assert_eq!(created_after, created_before);
        assert_eq!(accessed_after, accessed_before);
        assert_eq!(written_after_second_set, FT_2021_01_01_UTC);
    }

    #[test]
    fn returns_error_for_missing_file() {
        let path = unique_path("returns_error_for_missing_file");
        let _ = fs::remove_file(&path);

        let result = set_last_write_time(&path, filetime_from_raw(FT_2020_01_01_UTC));

        assert!(result.is_err());
    }
}