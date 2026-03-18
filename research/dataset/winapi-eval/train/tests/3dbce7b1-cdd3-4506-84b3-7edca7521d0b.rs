#[cfg(test)]
mod tests {
    use super::*;

    fn unique_pipe_name(tag: &str) -> String {
        use std::sync::atomic::{AtomicU64, Ordering};
        static COUNTER: AtomicU64 = AtomicU64::new(0);

        let n = COUNTER.fetch_add(1, Ordering::Relaxed);
        format!(
            r"\\.\pipe\{}_{}_{}_{}",
            tag,
            std::process::id(),
            std::thread::current().name().unwrap_or("t"),
            n
        )
    }

    #[test]
    fn test_named_pipe_impersonated_sid_happy_path() {
        let pipe = unique_pipe_name("imp_test");
        let sid = named_pipe_impersonated_sid(&pipe).unwrap();
        assert!(!sid.is_empty());
        assert!(sid.starts_with("S-"));
    }

    #[test]
    fn test_named_pipe_impersonated_sid_concurrent_access() {
        let handle = std::thread::spawn(|| {
            let pipe = unique_pipe_name("imp_test_concurrent");
            named_pipe_impersonated_sid(&pipe).unwrap()
        });

        let sid = handle.join().unwrap();
        assert!(!sid.is_empty());
        assert!(sid.starts_with("S-"));
    }

    #[test]
    fn test_named_pipe_impersonated_sid_multiple_clients() {
        let mut handles = vec![];
        for _ in 0..3 {
            handles.push(std::thread::spawn(|| {
                let pipe = unique_pipe_name("imp_test_multi");
                named_pipe_impersonated_sid(&pipe).unwrap()
            }));
        }

        let sids: Vec<String> = handles.into_iter().map(|h| h.join().unwrap()).collect();
        assert_eq!(sids.len(), 3);
        assert!(
            sids.iter()
                .all(|sid| !sid.is_empty() && sid.starts_with("S-"))
        );
    }
}