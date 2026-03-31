#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use windows::{
        core::PCWSTR,
        Win32::System::Threading::CreateEventW,
    };

    static NEXT_ID: AtomicUsize = AtomicUsize::new(1);

    fn unique_event_name(tag: &str) -> String {
        let id = NEXT_ID.fetch_add(1, Ordering::Relaxed);
        format!(r"Local\chatgpt.{}.{}.{}", tag, std::process::id(), id)
    }

    #[test]
    fn creates_signals_and_resets_a_new_named_event() {
        let name = unique_event_name("new");
        let snapshot = roundtrip_named_manual_reset_event(&name).unwrap();

        assert!(!snapshot.already_existed);
        assert!(snapshot.signaled_immediately);
        assert!(snapshot.reset_to_nonsignaled);
    }

    #[test]
    fn detects_when_the_named_event_already_exists() {
        let name = unique_event_name("existing");
        let wide_name = to_wide_null(&name).unwrap();

        let existing_handle =
            unsafe { CreateEventW(None, true, false, PCWSTR(wide_name.as_ptr())) }.unwrap();
        let _guard = HandleGuard(existing_handle);

        let snapshot = roundtrip_named_manual_reset_event(&name).unwrap();

        assert!(snapshot.already_existed);
        assert!(snapshot.signaled_immediately);
        assert!(snapshot.reset_to_nonsignaled);
    }

    #[test]
    fn rejects_names_with_interior_nul() {
        let result = roundtrip_named_manual_reset_event("bad\0name");
        assert!(result.is_err());
    }
}
