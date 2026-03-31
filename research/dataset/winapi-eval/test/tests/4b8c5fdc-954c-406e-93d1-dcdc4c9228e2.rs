#[cfg(test)]
mod tests {
    use super::set_process_env_var;
    use std::sync::atomic::{AtomicUsize, Ordering};

    static COUNTER: AtomicUsize = AtomicUsize::new(0);

    fn unique_name() -> String {
        format!(
            "RUST_WIN_API_TEST_{}_{}",
            std::process::id(),
            COUNTER.fetch_add(1, Ordering::Relaxed)
        )
    }

    #[test]
    fn creates_variable_visible_to_std_env() {
        let name = unique_name();

        set_process_env_var(&name, "alpha").unwrap();

        let value = std::env::var(&name).unwrap();
        assert_eq!(value, "alpha");
    }

    #[test]
    fn overwrites_existing_value() {
        let name = unique_name();

        set_process_env_var(&name, "first").unwrap();
        set_process_env_var(&name, "second").unwrap();

        let value = std::env::var(&name).unwrap();
        assert_eq!(value, "second");
    }
}
