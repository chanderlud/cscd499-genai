#[cfg(test)]
mod tests {
    use super::*;
    use std::{
        fs,
        path::{Path, PathBuf},
        process,
        time::{SystemTime, UNIX_EPOCH},
    };

    #[cfg(windows)]
    struct TempFile {
        path: PathBuf,
    }

    #[cfg(windows)]
    impl TempFile {
        fn new(name_hint: &str, contents: &[u8]) -> Self {
            let mut path = std::env::temp_dir();
            let nanos = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos();
            path.push(format!("{}_{}_{}.bin", name_hint, process::id(), nanos));
            fs::write(&path, contents).unwrap();
            Self { path }
        }

        fn path(&self) -> &Path {
            &self.path
        }
    }

    #[cfg(windows)]
    impl Drop for TempFile {
        fn drop(&mut self) {
            let _ = fs::remove_file(&self.path);
        }
    }

    #[cfg(windows)]
    fn nonexistent_temp_path(name_hint: &str) -> PathBuf {
        let mut path = std::env::temp_dir();
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        path.push(format!(
            "{}_{}_{}_does_not_exist.bin",
            name_hint,
            process::id(),
            nanos
        ));
        path
    }

    #[test]
    #[cfg(windows)]
    fn not_found_returns_error() {
        let p = nonexistent_temp_path("mmap_count_nf");
        let err = mmap_count_nonoverlapping(&p, b"aa").unwrap_err();
        assert_eq!(err.kind(), std::io::ErrorKind::NotFound);
    }

    #[test]
    #[cfg(windows)]
    fn empty_file_nonempty_needle_is_zero() {
        let f = TempFile::new("mmap_count_empty_file", b"");
        let out = mmap_count_nonoverlapping(f.path(), b"a").unwrap();
        assert_eq!(out, 0);
    }

    #[test]
    #[cfg(windows)]
    fn needle_longer_than_file_is_zero() {
        let f = TempFile::new("mmap_count_short", b"abc");
        let out = mmap_count_nonoverlapping(f.path(), b"abcd").unwrap();
        assert_eq!(out, 0);
    }

    #[test]
    #[cfg(windows)]
    fn needle_equals_file_is_one() {
        let f = TempFile::new("mmap_count_equal", b"pattern");
        let out = mmap_count_nonoverlapping(f.path(), b"pattern").unwrap();
        assert_eq!(out, 1);
    }

    #[test]
    #[cfg(windows)]
    fn single_byte_counts_every_occurrence() {
        let f = TempFile::new("mmap_count_single", b"bananas");
        let out = mmap_count_nonoverlapping(f.path(), b"a").unwrap();
        assert_eq!(out, 3);
    }

    #[test]
    #[cfg(windows)]
    fn non_overlapping_example_aaaaa_aa_is_two() {
        let f = TempFile::new("mmap_count_aaaaa", b"aaaaa");
        let out = mmap_count_nonoverlapping(f.path(), b"aa").unwrap();
        assert_eq!(out, 2);
    }

    #[test]
    #[cfg(windows)]
    fn overlapping_matches_must_not_be_counted() {
        let f = TempFile::new("mmap_count_overlap", b"ababa");
        let out = mmap_count_nonoverlapping(f.path(), b"aba").unwrap();
        assert_eq!(out, 1);
    }

    #[test]
    #[cfg(windows)]
    fn overlapping_like_aaaaa_aaa_is_one() {
        let f = TempFile::new("mmap_count_aaa", b"aaaaa");
        let out = mmap_count_nonoverlapping(f.path(), b"aaa").unwrap();
        assert_eq!(out, 1);
    }

    #[test]
    #[cfg(windows)]
    fn finds_multiple_non_overlapping_occurrences_spaced_out() {
        let f = TempFile::new("mmap_count_spaced", b"xx--xx--xx");
        let out = mmap_count_nonoverlapping(f.path(), b"xx").unwrap();
        assert_eq!(out, 3);
    }

    #[test]
    #[cfg(windows)]
    fn match_at_end_boundary_counts() {
        let f = TempFile::new("mmap_count_end", b"zzzEND");
        let out = mmap_count_nonoverlapping(f.path(), b"END").unwrap();
        assert_eq!(out, 1);
    }

    #[test]
    #[cfg(windows)]
    fn handles_embedded_nuls_binary_data() {
        let data = [0u8, 0, 1, 0, 0, 2, 0, 0];
        let f = TempFile::new("mmap_count_nul", &data);
        let out = mmap_count_nonoverlapping(f.path(), &[0, 0]).unwrap();
        assert_eq!(out, 3);
    }

    #[test]
    #[cfg(windows)]
    fn large_input_sanity_check() {
        let n: usize = 1024 * 1024;
        let data = vec![b'a'; n];
        let f = TempFile::new("mmap_count_large", &data);

        let out = mmap_count_nonoverlapping(f.path(), b"aaa").unwrap();
        assert_eq!(out, n / 3);
    }

    #[test]
    #[cfg(windows)]
    fn empty_needle_is_handled_safely() {
        let f = TempFile::new("mmap_count_empty_needle", b"anything");
        let res = mmap_count_nonoverlapping(f.path(), b"");
        match res {
            Ok(0) => {}
            Err(e) if e.kind() == std::io::ErrorKind::InvalidInput => {}
            Ok(other) => panic!("empty needle should not produce a positive count; got {other}"),
            Err(e) => panic!("unexpected error kind for empty needle: {e:?}"),
        }
    }
}
