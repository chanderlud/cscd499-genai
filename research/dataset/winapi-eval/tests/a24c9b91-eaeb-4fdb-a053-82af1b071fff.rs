#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicU64, Ordering};

    static NEXT_ID: AtomicU64 = AtomicU64::new(0);

    fn unique_name(prefix: &str) -> String {
        let seq = NEXT_ID.fetch_add(1, Ordering::Relaxed);
        format!("{prefix}_pid{}_{}", std::process::id(), seq)
    }

    fn unique_pair(prefix: &str) -> (String, String) {
        (
            unique_name(&format!("{prefix}_ns")),
            unique_name(&format!("{prefix}_mtx")),
        )
    }

    #[test]
    fn private_namespace_mutex_roundtrip_returns_true_for_fresh_names() {
        let (ns_name, mutex_name) = unique_pair("roundtrip");

        let ok = private_namespace_mutex_roundtrip(&ns_name, &mutex_name)
            .expect("expected namespace creation + mutex open roundtrip to succeed");

        assert!(
            ok,
            "expected OpenMutexW to reopen the mutex created inside the private namespace"
        );
    }

    #[test]
    fn private_namespace_mutex_roundtrip_supports_unicode_names() {
        let ns_name = format!("名前空間_{}", unique_name("unicode"));
        let mutex_name = format!("ミューテックス_ß_Ω_{}", unique_name("unicode"));

        let ok = private_namespace_mutex_roundtrip(&ns_name, &mutex_name)
            .expect("expected Unicode namespace and mutex names to work");

        assert!(ok, "expected Unicode roundtrip to succeed");
    }

    #[test]
    fn private_namespace_mutex_roundtrip_rejects_nul_in_namespace_name() {
        let mutex_name = unique_name("valid_mutex");

        let result = private_namespace_mutex_roundtrip("bad\0namespace", &mutex_name);

        assert!(
            result.is_err(),
            "embedded NUL in the namespace name should be rejected"
        );
    }

    #[test]
    fn private_namespace_mutex_roundtrip_rejects_backslash_in_mutex_name() {
        let ns_name = unique_name("valid_ns");

        let result = private_namespace_mutex_roundtrip(&ns_name, r"bad\mutex");

        assert!(
            result.is_err(),
            "mutex leaf names should reject backslashes because the namespace path already uses '\\'"
        );
    }
}