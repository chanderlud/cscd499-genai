#[cfg(test)]
mod tests {
    use super::named_pipe_frame_lengths;

    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::time::{SystemTime, UNIX_EPOCH};

    static COUNTER: AtomicUsize = AtomicUsize::new(0);

    fn unique_pipe_name(tag: &str) -> String {
        // Unique enough for parallel tests + multiple processes without needing rand.
        let pid = std::process::id();
        let ctr = COUNTER.fetch_add(1, Ordering::Relaxed);
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos();
        format!("{tag}_{pid}_{ctr}_{nanos}")
    }

    // --- Windows-only tests (named pipes are Win32) ---

    #[cfg(windows)]
    #[test]
    fn example_matches_spec() -> std::io::Result<()> {
        let pipe = unique_pipe_name("ipc_np_example");
        let frames = vec![b"a".to_vec(), b"bbbb".to_vec(), vec![0; 10]];
        let out = named_pipe_frame_lengths(&pipe, &frames)?;
        assert_eq!(out, vec![1, 4, 10]);
        Ok(())
    }

    #[cfg(windows)]
    #[test]
    fn empty_frames_round_trip() -> std::io::Result<()> {
        // Edge case: N = 0 should return an empty list and not block forever.
        let pipe = unique_pipe_name("ipc_np_empty");
        let frames: Vec<Vec<u8>> = vec![];
        let out = named_pipe_frame_lengths(&pipe, &frames)?;
        assert_eq!(out, Vec::<u32>::new());
        Ok(())
    }

    #[cfg(windows)]
    #[test]
    fn includes_zero_length_frames() -> std::io::Result<()> {
        // Edge case: frame length = 0 (valid, payload is absent).
        let pipe = unique_pipe_name("ipc_np_zero_len");
        let frames = vec![vec![], b"x".to_vec(), vec![], b"yz".to_vec()];
        let out = named_pipe_frame_lengths(&pipe, &frames)?;
        assert_eq!(out, vec![0, 1, 0, 2]);
        Ok(())
    }

    #[cfg(windows)]
    #[test]
    fn boundary_sizes_around_common_buffer_edges() -> std::io::Result<()> {
        // Stress partial reads/writes with sizes around 4K boundaries.
        // If the implementation does a single ReadFile/WriteFile and hopes, this tends to break.
        let pipe = unique_pipe_name("ipc_np_boundaries");

        let sizes: &[usize] = &[0, 1, 2, 3, 4, 4095, 4096, 4097, 8192, 65535];
        let frames: Vec<Vec<u8>> = sizes.iter().map(|&n| vec![0xA5; n]).collect();

        let out = named_pipe_frame_lengths(&pipe, &frames)?;
        let expected: Vec<u32> = sizes.iter().map(|&n| n as u32).collect();
        assert_eq!(out, expected);
        Ok(())
    }

    #[cfg(windows)]
    #[test]
    fn many_frames_small_payloads() -> std::io::Result<()> {
        // Many frames exercises repeated length-prefix parsing and response encoding/decoding.
        let pipe = unique_pipe_name("ipc_np_many");

        let mut frames = Vec::new();
        for i in 0..200usize {
            // Mix of empty and small frames.
            let len = (i % 17) as usize; // includes 0
            frames.push(vec![i as u8; len]);
        }

        let out = named_pipe_frame_lengths(&pipe, &frames)?;
        let expected: Vec<u32> = frames.iter().map(|f| f.len() as u32).collect();
        assert_eq!(out, expected);
        Ok(())
    }

    #[cfg(windows)]
    #[test]
    fn large_frame_forces_multiple_reads_writes() -> std::io::Result<()> {
        // Big payload basically forces chunking in pipe I/O (partial ReadFile/WriteFile).
        // If the implementation doesn't loop until complete, this tends to explode.
        let pipe = unique_pipe_name("ipc_np_large");

        let frames = vec![
            b"hi".to_vec(),
            vec![0x5A; 1_048_576 + 7], // ~1MB+ (not aligned, because humans love edge cases)
            vec![0u8; 123],
        ];

        let out = named_pipe_frame_lengths(&pipe, &frames)?;
        assert_eq!(out, vec![2, (1_048_576 + 7) as u32, 123]);
        Ok(())
    }

    #[cfg(windows)]
    #[test]
    fn reusing_same_pipe_name_sequentially_works() -> std::io::Result<()> {
        // Some implementations accidentally leave the server handle in a bad state,
        // or fail to clean up properly between runs.
        let pipe = unique_pipe_name("ipc_np_reuse");
        let frames1 = vec![b"a".to_vec(), b"bb".to_vec()];
        let out1 = named_pipe_frame_lengths(&pipe, &frames1)?;
        assert_eq!(out1, vec![1, 2]);

        let frames2 = vec![vec![9u8; 10], vec![]];
        let out2 = named_pipe_frame_lengths(&pipe, &frames2)?;
        assert_eq!(out2, vec![10, 0]);

        Ok(())
    }

    #[cfg(windows)]
    #[test]
    fn absurdly_long_pipe_name_errors() {
        // Windows has limits on named pipe path/name lengths. We don't care which error,
        // just that the function doesn't claim success.
        let base = unique_pipe_name("ipc_np_toolong");
        let very_long = format!("{base}_{}", "x".repeat(2000));
        let frames = vec![b"a".to_vec()];

        let res = named_pipe_frame_lengths(&very_long, &frames);
        assert!(
            res.is_err(),
            "expected error for excessively long pipe name"
        );
    }

    #[cfg(windows)]
    #[test]
    fn empty_pipe_name_returns_error() {
        let pipe_name = "";
        let frames = vec![b"a".to_vec()];
        let res = named_pipe_frame_lengths(pipe_name, &frames);
        assert!(res.is_err(), "expected error for empty pipe name");
    }

    #[cfg(windows)]
    #[test]
    fn pipe_name_containing_backslash_returns_error() {
        let pipe_name = r"pipe\name"; // Contains a backslash
        let frames = vec![b"a".to_vec()];
        let res = named_pipe_frame_lengths(pipe_name, &frames);
        assert!(
            res.is_err(),
            "expected error for pipe name containing backslash"
        );
    }
}
