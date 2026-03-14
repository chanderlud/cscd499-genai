#[cfg(test)]
mod tests {
    use super::named_pipe_fanout;

    use std::{
        sync::{
            atomic::{AtomicU64, Ordering},
            mpsc,
        },
        thread,
        time::{Duration, SystemTime, UNIX_EPOCH},
    };

    static COUNTER: AtomicU64 = AtomicU64::new(0);

    fn unique_pipe_name(base: &str) -> String {
        let pid = std::process::id();
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos();
        let c = COUNTER.fetch_add(1, Ordering::Relaxed);
        format!("{base}_{pid}_{nanos}_{c}")
    }

    fn run_with_timeout<T: Send + 'static>(
        dur: Duration,
        f: impl FnOnce() -> T + Send + 'static,
    ) -> T {
        let (tx, rx) = mpsc::channel();
        thread::spawn(move || {
            // If the function panics, nothing is sent and we time out (good enough for tests).
            let _ = tx.send(f());
        });
        rx.recv_timeout(dur).expect("timed out (likely deadlock)")
    }

    fn expected_vec(n: u32) -> Vec<u32> {
        (0..n).map(|i| i.wrapping_mul(3).wrapping_add(1)).collect()
    }

    #[test]
    fn fanout_basic_4() -> std::io::Result<()> {
        let name = unique_pipe_name("ipc_np_02_basic_4");
        let out = run_with_timeout(Duration::from_secs(10), move || named_pipe_fanout(&name, 4))?;
        assert_eq!(out, vec![1, 4, 7, 10]); // i*3+1 for i=0..3
        Ok(())
    }

    #[test]
    fn fanout_single_client() -> std::io::Result<()> {
        let name = unique_pipe_name("ipc_np_02_single");
        let out = run_with_timeout(Duration::from_secs(10), move || named_pipe_fanout(&name, 1))?;
        assert_eq!(out, vec![1]); // i=0 -> 1
        Ok(())
    }

    #[test]
    fn fanout_ordering_is_deterministic_32() -> std::io::Result<()> {
        let name = unique_pipe_name("ipc_np_02_order_32");
        let out = run_with_timeout(Duration::from_secs(15), move || named_pipe_fanout(&name, 32))?;
        assert_eq!(out, expected_vec(32)); // must be in client index order
        Ok(())
    }

    #[test]
    fn fanout_stress_reasonable_64_completes() -> std::io::Result<()> {
        // This isn't a benchmark, it's a "please don't deadlock" check.
        let name = unique_pipe_name("ipc_np_02_stress_64");
        let out = run_with_timeout(Duration::from_secs(20), move || named_pipe_fanout(&name, 64))?;
        assert_eq!(out, expected_vec(64));
        Ok(())
    }

    #[test]
    fn fanout_pipe_name_can_be_reused_back_to_back() -> std::io::Result<()> {
        // If you leak handles or leave pipe instances dangling, the second call often fails.
        let name = unique_pipe_name("ipc_np_02_reuse");

        let out1 =
            run_with_timeout(Duration::from_secs(10), {
                let name = name.clone();
                move || named_pipe_fanout(&name, 8)
            })?;
        assert_eq!(out1, expected_vec(8));

        // Give the OS a tiny moment to tear down handles if needed (helps reduce flakiness).
        thread::sleep(Duration::from_millis(50));

        let out2 = run_with_timeout(Duration::from_secs(10), move || named_pipe_fanout(&name, 8))?;
        assert_eq!(out2, expected_vec(8));

        Ok(())
    }

    #[test]
    fn fanout_zero_clients_is_handled_gracefully() {
        // Spec doesn't define this. We just refuse to let it panic or do something unhinged.
        let name = unique_pipe_name("ipc_np_02_zero");
        let res = run_with_timeout(Duration::from_secs(5), move || named_pipe_fanout(&name, 0));
        match res {
            Ok(v) => assert!(
                v.is_empty(),
                "if client_count=0 returns Ok, it should be an empty vec"
            ),
            Err(_) => {
                // Also acceptable: returning an error instead of trying to create 0 pipe instances.
            }
        }
    }

    #[test]
    fn fanout_empty_name_errors() {
        // An empty pipe name should not magically work.
        let res = run_with_timeout(Duration::from_secs(5), move || named_pipe_fanout("", 1));
        assert!(res.is_err(), "expected error for empty pipe name");
    }
}