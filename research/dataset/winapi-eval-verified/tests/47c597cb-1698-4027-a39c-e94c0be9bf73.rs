#[cfg(test)]
mod tests {
    use crate::shm_ringbuffer_roundtrip;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::sync::mpsc;
    use std::time::Duration;

    const TIMEOUT: Duration = Duration::from_secs(10);

    fn unique_base_name(prefix: &str) -> String {
        static CTR: AtomicUsize = AtomicUsize::new(0);
        let n = CTR.fetch_add(1, Ordering::Relaxed);
        format!("{}_{}_{}", prefix, std::process::id(), n)
    }

    fn run_with_timeout<F, T>(f: F) -> T
    where
        F: FnOnce() -> T + Send + 'static,
        T: Send + 'static,
    {
        let (tx, rx) = mpsc::channel();
        std::thread::spawn(move || {
            let _ = tx.send(f());
        });

        rx.recv_timeout(TIMEOUT)
            .unwrap_or_else(|_| panic!("timed out after {:?} (likely deadlock)", TIMEOUT))
    }

    fn roundtrip(capacity: u32, chunks: Vec<Vec<u8>>) -> Vec<u8> {
        let base = unique_base_name("ipc_shm_07_test");
        run_with_timeout(move || {
            shm_ringbuffer_roundtrip(&base, capacity, &chunks)
                .unwrap_or_else(|e| panic!("roundtrip returned error: {e}"))
        })
    }

    fn expected(chunks: &[Vec<u8>]) -> Vec<u8> {
        chunks.concat()
    }

    #[test]
    fn example_case_wraps_and_matches() {
        let chunks = vec![b"abc".to_vec(), vec![0; 100], b"xyz".to_vec()];
        let out = roundtrip(64, chunks.clone());
        assert_eq!(out, expected(&chunks));
    }

    #[test]
    fn empty_input_returns_empty() {
        let chunks: Vec<Vec<u8>> = vec![];
        let out = roundtrip(8, chunks.clone());
        assert_eq!(out, expected(&chunks));
        assert!(out.is_empty());
    }

    #[test]
    fn handles_empty_chunks_in_between() {
        let chunks = vec![
            vec![],
            b"hello".to_vec(),
            vec![],
            b"".to_vec(),
            b"world".to_vec(),
            vec![],
        ];
        let out = roundtrip(16, chunks.clone());
        assert_eq!(out, expected(&chunks));
    }

    #[test]
    fn many_small_chunks_tiny_capacity_forces_wraparound_often() {
        // Capacity is intentionally tiny to force frequent wrap-around and partial writes/reads.
        let chunks = vec![
            b"a".to_vec(),
            b"bb".to_vec(),
            b"ccc".to_vec(),
            b"dddd".to_vec(),
            b"eeeee".to_vec(),
            b"ffffff".to_vec(),
            b"ggggggg".to_vec(),
            b"hhhhhhhh".to_vec(),
            b"i".to_vec(),
            b"jk".to_vec(),
        ];
        let out = roundtrip(8, chunks.clone());
        assert_eq!(out, expected(&chunks));
    }

    #[test]
    fn single_chunk_much_larger_than_capacity() {
        // Forces the implementation to stream through the ring buffer in slices, not per-byte.
        let big: Vec<u8> = (0..10_000u32).map(|i| (i % 256) as u8).collect();
        let chunks = vec![big.clone()];
        let out = roundtrip(64, chunks.clone());
        assert_eq!(out.len(), big.len());
        assert_eq!(out, big);
    }

    #[test]
    fn mixed_sizes_prime_capacity_crosses_boundaries_weirdly() {
        // Prime capacity helps shake out incorrect modulo / boundary logic assumptions.
        let mut chunks = Vec::new();
        chunks.push(vec![1u8; 17]);
        chunks.push(vec![2u8; 5]);
        chunks.push(vec![3u8; 29]);
        chunks.push(vec![4u8; 1]);
        chunks.push((0..255u16).map(|i| i as u8).collect::<Vec<u8>>());
        chunks.push(vec![9u8; 64]);
        chunks.push(b"tail".to_vec());

        let out = roundtrip(17, chunks.clone());
        assert_eq!(out, expected(&chunks));
    }

    #[test]
    fn total_size_multiple_of_capacity_does_not_corrupt_or_drop_bytes() {
        // This can expose “read==write means empty” bugs if full/empty logic is wrong during cycles.
        let capacity = 32;
        let chunks = vec![vec![7u8; 64], vec![8u8; 96]]; // total = 160 = 5 * 32
        let out = roundtrip(capacity, chunks.clone());
        assert_eq!(out, expected(&chunks));
    }

    #[test]
    fn repeated_roundtrips_with_same_inputs_are_deterministic() {
        let chunks = vec![
            b"det".to_vec(),
            b"ermin".to_vec(),
            b"istic".to_vec(),
            vec![0; 123],
        ];
        let expect = expected(&chunks);

        let out1 = roundtrip(24, chunks.clone());
        let out2 = roundtrip(24, chunks.clone());

        assert_eq!(out1, expect);
        assert_eq!(out2, expect);
        assert_eq!(out1, out2);
    }
}
