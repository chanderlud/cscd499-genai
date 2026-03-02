#[cfg(all(test, windows))]
mod tests {
    use super::get_environment_kv;
    use std::io;
    use std::sync::{Mutex, OnceLock};

    // Tests mutate process-wide environment variables; Rust runs tests in parallel by default.
    // So we serialize them to avoid flaky nonsense.
    fn env_lock() -> &'static Mutex<()> {
        static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
        LOCK.get_or_init(|| Mutex::new(()))
    }

    unsafe fn set_var_scoped<K: AsRef<str>>(key: K, value: &str) -> impl Drop {
        struct Guard {
            key: String,
            old: Option<std::ffi::OsString>,
        }
        impl Drop for Guard {
            fn drop(&mut self) {
                unsafe {
                    match self.old.take() {
                        Some(v) => std::env::set_var(&self.key, v),
                        None => std::env::remove_var(&self.key),
                    }
                }
            }
        }

        let key = key.as_ref().to_string();
        let old = std::env::var_os(&key);
        std::env::set_var(&key, value);
        Guard { key, old }
    }

    fn find<'a>(env: &'a [(String, String)], key: &str) -> Option<&'a str> {
        env.iter()
            .find(|(k, _)| k.eq_ignore_ascii_case(key))
            .map(|(_, v)| v.as_str())
    }

    #[test]
    fn includes_newly_set_var() -> io::Result<()> {
        let _g = env_lock().lock().unwrap();

        let key = "RUST_WINAPI_TEST_VAR";
        let _guard = unsafe { set_var_scoped(key, "xyz") };

        let env = get_environment_kv()?;
        assert!(
            env.iter().any(|(k, v)| k == key && v == "xyz"),
            "Expected ({key}=xyz) to appear in environment block"
        );
        Ok(())
    }

    #[test]
    fn supports_empty_value() -> io::Result<()> {
        let _g = env_lock().lock().unwrap();

        let key = "RUST_WINAPI_TEST_EMPTY_VALUE";
        let _guard = unsafe { set_var_scoped(key, "") };

        let env = get_environment_kv()?;
        assert!(
            env.iter().any(|(k, v)| k == key && v.is_empty()),
            "Expected ({key}=) to appear with an empty value"
        );
        Ok(())
    }

    #[test]
    fn value_can_contain_equals() -> io::Result<()> {
        let _g = env_lock().lock().unwrap();

        let key = "RUST_WINAPI_TEST_EQUALS_IN_VALUE";
        let value = "a=b=c";
        let _guard = unsafe { set_var_scoped(key, value) };

        let env = get_environment_kv()?;
        assert_eq!(
            find(&env, key),
            Some(value),
            "Parser should split on the correct '=' and keep the rest in the value"
        );
        Ok(())
    }

    #[test]
    fn utf16_to_utf8_roundtrip_for_unicode_value() -> io::Result<()> {
        let _g = env_lock().lock().unwrap();

        let key = "RUST_WINAPI_TEST_UNICODE_VALUE";
        let value = "café ☕ 你好 🦀";
        let _guard = unsafe { set_var_scoped(key, value) };

        let env = get_environment_kv()?;
        assert_eq!(
            find(&env, key),
            Some(value),
            "Expected Unicode value to survive UTF-16 -> UTF-8 conversion"
        );
        Ok(())
    }

    #[test]
    fn handles_large_value() -> io::Result<()> {
        let _g = env_lock().lock().unwrap();

        let key = "RUST_WINAPI_TEST_LARGE_VALUE";
        // Keep comfortably under Windows' 32,767 char limit for a single env var.
        let value = "x".repeat(20_000);
        let _guard = unsafe { set_var_scoped(key, &value) };

        let env = get_environment_kv()?;
        let got = find(&env, key).expect("Expected large env var to be present");
        assert_eq!(got.len(), value.len(), "Large value length mismatch");
        assert_eq!(got, value, "Large value contents mismatch");
        Ok(())
    }

    #[test]
    fn no_empty_keys_and_no_embedded_nuls() -> io::Result<()> {
        let _g = env_lock().lock().unwrap();

        let env = get_environment_kv()?;

        // A classic Windows gotcha: environment block can include special entries like "=C:=C:\path".
        // A naive split-on-first-'=' yields an empty key. That's wrong. Keys should never be empty.
        assert!(
            env.iter().all(|(k, _)| !k.is_empty()),
            "Found an empty key; likely mis-parsed entries that start with '=' (e.g. '=C:=...')"
        );

        // Sanity: after parsing a double-NUL-terminated block, no String should contain '\0'.
        assert!(
            env.iter().all(|(k, v)| !k.contains('\0') && !v.contains('\0')),
            "Found embedded NUL in parsed key/value; parser likely mishandled termination"
        );

        Ok(())
    }

    #[test]
    fn repeated_calls_are_stable() -> io::Result<()> {
        let _g = env_lock().lock().unwrap();

        // Not a perfect leak detector, but it will catch some obvious misuse like returning
        // references into freed memory or other “exciting” pointer bugs.
        for _ in 0..50 {
            let env = get_environment_kv()?;
            assert!(
                !env.is_empty(),
                "Environment should generally not be empty; parser may have stopped early"
            );
        }

        Ok(())
    }
}