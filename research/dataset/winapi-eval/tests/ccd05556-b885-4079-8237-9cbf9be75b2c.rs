#[cfg(test)]
mod tests {
    use super::*;

    use std::{
        env,
        fs::{self, File},
        io::Write,
        path::{Path, PathBuf},
        process,
        sync::atomic::{AtomicUsize, Ordering},
    };

    static NEXT_ID: AtomicUsize = AtomicUsize::new(0);

    struct TestFile {
        path: PathBuf,
    }

    impl TestFile {
        fn path(&self) -> &Path {
            &self.path
        }
    }

    impl Drop for TestFile {
        fn drop(&mut self) {
            let _ = fs::remove_file(&self.path);
        }
    }

    fn make_test_file(bytes: &[u8]) -> TestFile {
        let dir = env::temp_dir().join("read_file_iocp_tests");
        fs::create_dir_all(&dir).expect("failed to create test temp directory");

        let id = NEXT_ID.fetch_add(1, Ordering::Relaxed);
        let path = dir.join(format!(
            "read_file_iocp_{}_{}.bin",
            process::id(),
            id
        ));

        let mut file = File::create(&path).expect("failed to create test file");
        file.write_all(bytes).expect("failed to write test file");
        file.flush().expect("failed to flush test file");
        drop(file);

        TestFile { path }
    }

    fn patterned_bytes(len: usize) -> Vec<u8> {
        // Nontrivial deterministic pattern so ordering bugs are obvious.
        (0..len)
            .map(|i| ((i.wrapping_mul(31) + i / 7 + 13) % 251) as u8)
            .collect()
    }

    #[test]
    fn reads_entire_file_with_many_in_flight_reads_and_partial_tail() {
        let expected = patterned_bytes(1_048_576 + 12_345);
        let file = make_test_file(&expected);

        let actual = read_file_iocp(file.path(), 64 * 1024, 8)
            .expect("read_file_iocp should succeed");

        assert_eq!(actual.len(), expected.len(), "returned length mismatch");
        assert_eq!(actual, expected, "returned bytes were not in exact file order");
    }

    #[test]
    fn reads_small_file_when_chunk_size_exceeds_file_len() {
        let expected = patterned_bytes(37);
        let file = make_test_file(&expected);

        let actual = read_file_iocp(file.path(), 64 * 1024, 8)
            .expect("read_file_iocp should succeed for a small file");

        assert_eq!(actual, expected);
    }

    #[test]
    fn reads_empty_file() {
        let expected = Vec::<u8>::new();
        let file = make_test_file(&expected);

        let actual = read_file_iocp(file.path(), 4096, 4)
            .expect("read_file_iocp should succeed for an empty file");

        assert!(actual.is_empty(), "empty file should return an empty Vec");
    }

    #[test]
    fn reads_correctly_when_max_in_flight_exceeds_number_of_chunks() {
        let expected = patterned_bytes((3 * 4096) + 123);
        let file = make_test_file(&expected);

        let actual = read_file_iocp(file.path(), 4096, 64)
            .expect("read_file_iocp should succeed when max_in_flight exceeds chunk count");

        assert_eq!(actual, expected);
    }
}