#[cfg(test)]
mod tests {
    use super::*;

    use std::{
        fs::{self, File},
        path::{Path, PathBuf},
        sync::{
            atomic::{AtomicBool, Ordering},
            Arc,
        },
        thread,
        time::{Duration, SystemTime, UNIX_EPOCH},
    };

    struct TestDir {
        path: PathBuf,
    }

    impl TestDir {
        fn new(test_name: &str) -> Self {
            let unique = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos();

            let path = std::env::temp_dir().join(format!(
                "wait_for_create_{test_name}_{}_{}",
                std::process::id(),
                unique
            ));

            fs::create_dir_all(&path).unwrap();
            Self { path }
        }

        fn path(&self) -> &Path {
            &self.path
        }
    }

    impl Drop for TestDir {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.path);
        }
    }

    fn touch(path: &Path) {
        File::create(path).unwrap();
    }

    #[test]
    fn times_out_when_no_file_is_created() {
        let dir = TestDir::new("times_out");

        let observed = wait_for_create(dir.path(), 200).unwrap();

        assert!(
            observed.is_none(),
            "expected timeout with no create event, got {observed:?}"
        );
    }

    #[test]
    fn returns_some_created_file_name() {
        let dir = TestDir::new("returns_created_name");

        let stop = Arc::new(AtomicBool::new(false));
        let stop_for_thread = Arc::clone(&stop);
        let dir_for_thread = dir.path().to_path_buf();

        let creator = thread::spawn(move || {
            // Repeated creates make this deterministic without relying on a
            // single fragile timing window.
            for i in 0..64 {
                if stop_for_thread.load(Ordering::Acquire) {
                    break;
                }

                let path = dir_for_thread.join(format!("created_{i}.txt"));
                touch(&path);
                thread::sleep(Duration::from_millis(15));
            }
        });

        let observed = wait_for_create(dir.path(), 2_000).unwrap();

        stop.store(true, Ordering::Release);
        creator.join().unwrap();

        let observed = observed.expect("expected a create event before timeout");
        let observed = observed.to_string_lossy();

        assert!(
            observed.starts_with("created_") && observed.ends_with(".txt"),
            "unexpected created file name: {observed}"
        );
    }

    #[test]
    fn does_not_report_preexisting_files() {
        let dir = TestDir::new("ignores_preexisting");

        touch(&dir.path().join("already_here.txt"));

        let observed = wait_for_create(dir.path(), 200).unwrap();

        assert!(
            observed.is_none(),
            "preexisting files must not count as newly created: {observed:?}"
        );
    }

    #[test]
    fn waits_for_a_new_file_even_if_directory_already_contains_files() {
        let dir = TestDir::new("waits_for_new_file");

        touch(&dir.path().join("already_here.txt"));

        let stop = Arc::new(AtomicBool::new(false));
        let stop_for_thread = Arc::clone(&stop);
        let dir_for_thread = dir.path().to_path_buf();

        let creator = thread::spawn(move || {
            for i in 0..64 {
                if stop_for_thread.load(Ordering::Acquire) {
                    break;
                }

                let path = dir_for_thread.join(format!("new_file_{i}.txt"));
                touch(&path);
                thread::sleep(Duration::from_millis(15));
            }
        });

        let observed = wait_for_create(dir.path(), 2_000).unwrap();

        stop.store(true, Ordering::Release);
        creator.join().unwrap();

        let observed = observed.expect("expected a new create event before timeout");
        let observed = observed.to_string_lossy();

        assert_ne!(observed, "already_here.txt");
        assert!(
            observed.starts_with("new_file_") && observed.ends_with(".txt"),
            "unexpected created file name: {observed}"
        );
    }
}