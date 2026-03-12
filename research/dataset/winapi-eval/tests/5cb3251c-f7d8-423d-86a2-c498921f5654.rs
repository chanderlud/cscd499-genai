#[cfg(test)]
mod tests {
    use super::shm_interlocked_counter;

    fn expected(threads: usize, iters: i64) -> i64 {
        (threads as i64) * iters
    }

    #[test]
    fn example_matches_spec() -> std::io::Result<()> {
        let out = shm_interlocked_counter(8, 10_000)?;
        assert_eq!(out, 80_000);
        Ok(())
    }

    #[test]
    fn single_thread_is_correct() -> std::io::Result<()> {
        let out = shm_interlocked_counter(1, 123_456)?;
        assert_eq!(out, 123_456);
        Ok(())
    }

    #[test]
    fn zero_iters_returns_zero() -> std::io::Result<()> {
        let out = shm_interlocked_counter(16, 0)?;
        assert_eq!(out, 0);
        Ok(())
    }

    #[test]
    fn zero_threads_returns_zero() -> std::io::Result<()> {
        let out = shm_interlocked_counter(0, 50_000)?;
        assert_eq!(out, 0);
        Ok(())
    }

    #[test]
    fn deterministic_over_repeats() -> std::io::Result<()> {
        let threads = 8usize;
        let iters = 25_000i64;
        let want = expected(threads, iters);

        for _ in 0..5 {
            let out = shm_interlocked_counter(threads, iters)?;
            assert_eq!(out, want);
        }
        Ok(())
    }

    #[test]
    fn stress_high_contention_catches_non_atomic_bugs() -> std::io::Result<()> {
        let hw = std::thread::available_parallelism()
            .map(|n| n.get())
            .unwrap_or(4);
        let threads = (hw * 2).clamp(4, 32);
        let iters = 50_000i64;

        let out = shm_interlocked_counter(threads, iters)?;
        assert_eq!(out, expected(threads, iters));
        Ok(())
    }

    #[test]
    fn concurrent_calls_do_not_interfere_with_each_other() -> std::io::Result<()> {
        let (t1, i1) = (6usize, 40_000i64);
        let (t2, i2) = (9usize, 15_000i64);

        let r = std::thread::scope(|s| {
            let h1 = s.spawn(|| shm_interlocked_counter(t1, i1));
            let h2 = s.spawn(|| shm_interlocked_counter(t2, i2));
            (h1.join().unwrap(), h2.join().unwrap())
        });

        let out1 = r.0?;
        let out2 = r.1?;

        assert_eq!(out1, expected(t1, i1));
        assert_eq!(out2, expected(t2, i2));
        Ok(())
    }

    #[test]
    fn many_small_calls_no_state_leak() -> std::io::Result<()> {
        for k in 1..=25usize {
            let threads = (k % 8) + 1;
            let iters = (k as i64) * 1_000;
            let out = shm_interlocked_counter(threads, iters)?;
            assert_eq!(out, expected(threads, iters));
        }
        Ok(())
    }
}
