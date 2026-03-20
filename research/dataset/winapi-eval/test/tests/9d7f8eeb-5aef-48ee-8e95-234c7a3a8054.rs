#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::path::PathBuf;
    use std::sync::atomic::{AtomicUsize, Ordering};

    static NEXT_ID: AtomicUsize = AtomicUsize::new(0);

    fn unique_ini_path(prefix: &str) -> PathBuf {
        let id = NEXT_ID.fetch_add(1, Ordering::Relaxed);
        let path =
            std::env::temp_dir().join(format!("{}_{}_{}.ini", prefix, std::process::id(), id));
        let _ = fs::remove_file(&path);
        path
    }

    #[test]
    fn creates_file_and_roundtrips_first_entry() {
        let path = unique_ini_path("airlock_log");

        let snapshot = stamp_airlock_log(&path, "Dock-7", "badge-042").unwrap();

        assert_eq!(snapshot.cycles, 1);
        assert_eq!(snapshot.last_badge, "badge-042");
        assert!(path.exists());

        let text = fs::read_to_string(&path).unwrap();
        assert!(text.contains("[Dock-7]"));
        assert!(text.contains("last_badge=badge-042"));
        assert!(text.contains("cycles=1"));

        let _ = fs::remove_file(path);
    }

    #[test]
    fn increments_counter_and_updates_badge() {
        let path = unique_ini_path("airlock_log");

        let first = stamp_airlock_log(&path, "Dock-9", "alpha").unwrap();
        let second = stamp_airlock_log(&path, "Dock-9", "beta").unwrap();

        assert_eq!(first.cycles, 1);
        assert_eq!(second.cycles, 2);
        assert_eq!(second.last_badge, "beta");

        let text = fs::read_to_string(&path).unwrap();
        assert!(text.contains("[Dock-9]"));
        assert!(text.contains("last_badge=beta"));
        assert!(text.contains("cycles=2"));

        let _ = fs::remove_file(path);
    }
}
