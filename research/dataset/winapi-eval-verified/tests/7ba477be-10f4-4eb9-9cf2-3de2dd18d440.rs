#[cfg(all(test, windows))]
mod tests {
    use crate::named_pipe_uppercase_echo;

    use std::sync::{
        atomic::{AtomicUsize, Ordering},
        mpsc,
    };
    use std::thread;
    use std::time::Duration;

    static PIPE_COUNTER: AtomicUsize = AtomicUsize::new(0);

    fn unique_pipe_name(prefix: &str) -> String {
        // Keep it short to avoid name-length nonsense.
        let pid = std::process::id();
        let n = PIPE_COUNTER.fetch_add(1, Ordering::Relaxed);
        format!("{prefix}_{pid}_{n}")
    }

    fn ascii_uppercase_only(s: &str) -> String {
        s.chars()
            .map(|c| {
                if c.is_ascii_lowercase() {
                    c.to_ascii_uppercase()
                } else {
                    c
                }
            })
            .collect()
    }

    fn call_with_timeout(
        pipe_name: String,
        msg: String,
        timeout: Duration,
    ) -> std::io::Result<String> {
        let (tx, rx) = mpsc::channel();

        thread::spawn(move || {
            let res = named_pipe_uppercase_echo(&pipe_name, &msg);
            // If the receiver is gone, whatever. This is a test.
            let _ = tx.send(res);
        });

        match rx.recv_timeout(timeout) {
            Ok(res) => res,
            Err(mpsc::RecvTimeoutError::Timeout) => {
                panic!("named_pipe_uppercase_echo hung longer than {:?}", timeout);
            }
            Err(mpsc::RecvTimeoutError::Disconnected) => {
                panic!("worker thread died without returning a result");
            }
        }
    }

    #[test]
    fn basic_example() -> std::io::Result<()> {
        let pipe = unique_pipe_name("ipc_np_basic");
        let out = call_with_timeout(pipe, "Hello, world!".to_string(), Duration::from_secs(3))?;
        assert_eq!(out, "HELLO, WORLD!");
        Ok(())
    }

    #[test]
    fn empty_message_round_trip() -> std::io::Result<()> {
        let pipe = unique_pipe_name("ipc_np_empty");
        let out = call_with_timeout(pipe, "".to_string(), Duration::from_secs(3))?;
        assert_eq!(out, "");
        Ok(())
    }

    #[test]
    fn only_ascii_a_to_z_uppercased_unicode_preserved() -> std::io::Result<()> {
        // Includes:
        // - accented letters that must NOT be altered
        // - ß which Unicode uppercasing would turn into "SS" (not allowed by spec)
        // - dotless ı which must NOT become 'I' (only ASCII a-z)
        let msg = "héllö 世界 ß Straße ıi";
        let expected = ascii_uppercase_only(msg);

        let pipe = unique_pipe_name("ipc_np_unicode");
        let out = call_with_timeout(pipe, msg.to_string(), Duration::from_secs(3))?;
        assert_eq!(out, expected);
        Ok(())
    }

    #[test]
    fn embedded_nul_is_preserved() -> std::io::Result<()> {
        // Rust strings can contain NUL. If someone treats it like a C-string, this will break.
        let msg = "a\0b\0c";
        let expected = ascii_uppercase_only(msg);

        let pipe = unique_pipe_name("ipc_np_nul");
        let out = call_with_timeout(pipe, msg.to_string(), Duration::from_secs(3))?;
        assert_eq!(out.len(), msg.len());
        assert_eq!(out, expected);
        Ok(())
    }

    #[test]
    fn long_message_not_truncated() -> std::io::Result<()> {
        // Large enough to catch single-buffer ReadFile implementations and
        // ERROR_MORE_DATA handling issues in message mode.
        let chunk = "abCDefghijKLMNopqrSTUVwxyz-123_";
        let msg = chunk.repeat(600); // ~ 20k+ bytes

        let expected = ascii_uppercase_only(&msg);
        let pipe = unique_pipe_name("ipc_np_long");

        let out = call_with_timeout(pipe, msg, Duration::from_secs(8))?;
        assert_eq!(out.len(), expected.len());
        assert_eq!(out, expected);
        Ok(())
    }

    #[test]
    fn repeated_calls_reuse_same_pipe_name() -> std::io::Result<()> {
        // Ensures proper cleanup (handles closed, name reusable).
        let pipe = unique_pipe_name("ipc_np_reuse");

        let out1 = call_with_timeout(pipe.clone(), "first".to_string(), Duration::from_secs(3))?;
        let out2 = call_with_timeout(pipe.clone(), "Second".to_string(), Duration::from_secs(3))?;

        assert_eq!(out1, "FIRST");
        assert_eq!(out2, "SECOND");
        Ok(())
    }

    #[test]
    fn invalid_pipe_name_errors() {
        // Empty name should not magically work.
        // If it does, your implementation is “creative” in the bad way.
        let res = call_with_timeout("".to_string(), "x".to_string(), Duration::from_secs(3));
        assert!(
            res.is_err(),
            "expected error for empty pipe name, got {res:?}"
        );
    }

    #[test]
    fn concurrent_calls_dont_interfere() -> std::io::Result<()> {
        // Run multiple independent pipe exchanges in parallel.
        // This catches shared-state bugs and some timing hazards.
        let n = 8usize;
        let (tx, rx) = mpsc::channel();

        for i in 0..n {
            let tx = tx.clone();
            let pipe = unique_pipe_name("ipc_np_concurrent");
            let msg = format!("m{i}: aZ-ß-{}-end", "x".repeat(1000));

            thread::spawn(move || {
                let expected = ascii_uppercase_only(&msg);
                let res = named_pipe_uppercase_echo(&pipe, &msg).map(|out| (out, expected));
                let _ = tx.send(res);
            });
        }
        drop(tx);

        for _ in 0..n {
            let res = rx
                .recv_timeout(Duration::from_secs(10))
                .expect("worker timed out or died");
            let (out, expected) = res?;
            assert_eq!(out, expected);
        }

        Ok(())
    }
}
