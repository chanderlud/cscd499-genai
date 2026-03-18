#[cfg(test)]
mod tests {
    use super::*;

    use std::ffi::OsStr;
    use std::process::Command;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::thread;
    use std::time::{Duration, Instant};

    static NEXT_ID: AtomicUsize = AtomicUsize::new(0);

    struct TestKey {
        path: String,
    }

    impl TestKey {
        fn new() -> Result<Self> {
            let id = NEXT_ID.fetch_add(1, Ordering::Relaxed);
            let path = format!(
                r"Software\MyApp\Tests\wait_for_reg_change_hkcu_{}_{}",
                std::process::id(),
                id
            );

            reg_add_key(&path)?;
            Ok(Self { path })
        }
    }

    impl Drop for TestKey {
        fn drop(&mut self) {
            let _ = Command::new("reg")
                .args(["delete", &format!(r"HKCU\{}", self.path), "/f"])
                .output();
        }
    }

    fn run_reg<I, S>(args: I) -> Result<()>
    where
        I: IntoIterator<Item = S>,
        S: AsRef<OsStr>,
    {
        let output = Command::new("reg").args(args).output()?;
        if output.status.success() {
            return Ok(());
        }

        Err(std::io::Error::new(
            std::io::ErrorKind::Other,
            format!(
                "reg command failed: status={:?}, stdout={}, stderr={}",
                output.status.code(),
                String::from_utf8_lossy(&output.stdout),
                String::from_utf8_lossy(&output.stderr),
            ),
        )
        .into())
    }

    fn reg_add_key(path: &str) -> Result<()> {
        run_reg(["add", &format!(r"HKCU\{}", path), "/f"])
    }

    fn reg_set_sz(path: &str, name: &str, value: &str) -> Result<()> {
        run_reg([
            "add",
            &format!(r"HKCU\{}", path),
            "/v",
            name,
            "/t",
            "REG_SZ",
            "/d",
            value,
            "/f",
        ])
    }

    #[test]
    fn wait_for_reg_change_hkcu_returns_true_when_key_changes() -> Result<()> {
        let key = TestKey::new()?;
        reg_set_sz(&key.path, "Value", "before")?;

        let wait_path = key.path.clone();
        let waiter = thread::spawn(move || wait_for_reg_change_hkcu(&wait_path, 2_000));

        // Give the waiter thread time to open the key and arm the notification.
        thread::sleep(Duration::from_millis(150));

        reg_set_sz(&key.path, "Value", "after")?;

        let changed = waiter.join().expect("waiter thread panicked")?;
        assert!(changed, "expected a registry change to be observed");

        Ok(())
    }

    #[test]
    fn wait_for_reg_change_hkcu_returns_false_on_timeout_without_change() -> Result<()> {
        let key = TestKey::new()?;
        reg_set_sz(&key.path, "Value", "stable")?;

        let start = Instant::now();
        let changed = wait_for_reg_change_hkcu(&key.path, 200)?;
        let elapsed = start.elapsed();

        assert!(!changed, "expected timeout without any registry change");
        assert!(
            elapsed >= Duration::from_millis(100),
            "returned too early: waited only {:?}",
            elapsed
        );

        Ok(())
    }
}
