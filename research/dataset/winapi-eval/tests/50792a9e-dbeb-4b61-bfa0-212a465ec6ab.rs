#[cfg(test)]
mod tests {
    use super::*;

    use std::env;
    use std::fs::{self, File};
    use std::io::{Read, Seek, SeekFrom};
    use std::path::{Path, PathBuf};
    use std::process;
    use std::sync::atomic::{AtomicUsize, Ordering};

    static NEXT_FILE_ID: AtomicUsize = AtomicUsize::new(0);

    struct TempFile {
        path: PathBuf,
    }

    impl TempFile {
        fn new(name: &str) -> Self {
            let id = NEXT_FILE_ID.fetch_add(1, Ordering::Relaxed);
            let path = env::temp_dir().join(format!(
                "sparse_file_stats_{}_{}_{}.bin",
                name,
                process::id(),
                id,
            ));
            let _ = fs::remove_file(&path);
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

    fn read_exact_at(file: &mut File, offset: u64, buf: &mut [u8]) -> std::io::Result<()> {
        file.seek(SeekFrom::Start(offset))?;
        file.read_exact(buf)
    }

    #[test]
    fn sparse_file_stats_reports_logical_eof_and_sparse_allocation() -> Result<()> {
        let tmp = TempFile::new("tail_only");
        let logical_size = 64 << 20;
        let tail = b"x";

        let (eof, alloc) = sparse_file_stats(
            tmp.path(),
            logical_size,
            0,
            logical_size - tail.len() as u64,
            tail,
        )?;

        assert_eq!(eof, logical_size);
        assert_eq!(fs::metadata(tmp.path())?.len(), logical_size);

        assert!(
            alloc >= tail.len() as u64,
            "allocation size {alloc} must account for the written tail",
        );
        assert!(
            alloc < eof,
            "allocation size {alloc} should stay below EOF {eof} for a sparse file",
        );

        let mut file = File::open(tmp.path())?;

        let mut head = [0u8; 4096];
        read_exact_at(&mut file, 0, &mut head)?;
        assert!(
            head.iter().all(|&b| b == 0),
            "punched region should read back as zeros",
        );

        let mut got_tail = [0u8; 1];
        read_exact_at(&mut file, logical_size - 1, &mut got_tail)?;
        assert_eq!(&got_tail, tail);

        Ok(())
    }

    #[test]
    fn sparse_file_stats_keeps_tail_data_and_zeroes_interior_hole() -> Result<()> {
        let tmp = TempFile::new("interior_hole");
        let logical_size = 8 << 20;
        let hole_start = 1 << 20;
        let hole_len = 6 << 20;
        let tail = b"tail";

        let (eof, alloc) = sparse_file_stats(tmp.path(), logical_size, hole_start, hole_len, tail)?;

        assert_eq!(eof, logical_size);
        assert_eq!(fs::metadata(tmp.path())?.len(), logical_size);

        assert!(
            alloc >= tail.len() as u64,
            "allocation size {alloc} must include the tail block",
        );
        assert!(
            alloc < eof,
            "allocation size {alloc} should remain below EOF {eof} when sparse ranges were used",
        );

        let mut file = File::open(tmp.path())?;

        let mut hole_bytes = [0u8; 64];
        let hole_probe = hole_start + (hole_len / 2);
        read_exact_at(&mut file, hole_probe, &mut hole_bytes)?;
        assert!(
            hole_bytes.iter().all(|&b| b == 0),
            "bytes inside the punched hole should read back as zeros",
        );

        let mut got_tail = [0u8; 4];
        read_exact_at(
            &mut file,
            logical_size - tail.len() as u64,
            &mut got_tail,
        )?;
        assert_eq!(&got_tail, tail);

        Ok(())
    }
}