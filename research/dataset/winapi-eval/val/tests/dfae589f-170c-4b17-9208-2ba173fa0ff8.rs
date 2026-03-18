#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use windows::Win32::Foundation::CloseHandle;

    fn unique_mutex_name() -> String {
        static NEXT_ID: AtomicUsize = AtomicUsize::new(0);
        let id = NEXT_ID.fetch_add(1, Ordering::Relaxed);
        format!("Global\\create_named_mutex_test_{}_{}", std::process::id(), id)
    }

    #[test]
    fn first_call_reports_created_second_reports_already_exists() -> Result<()> {
        let name = unique_mutex_name();

        let (h1, created1) = create_named_mutex(&name)?;
        assert!(!h1.is_invalid(), "first handle should be valid");
        assert!(
            created1,
            "first CreateMutexW call should report that it created the mutex"
        );

        let (h2, created2) = create_named_mutex(&name)?;
        assert!(!h2.is_invalid(), "second handle should be valid");
        assert!(
            !created2,
            "second CreateMutexW call should report that the mutex already exists"
        );

        unsafe {
            assert!(CloseHandle(h2).is_ok(), "failed to close second handle");
            assert!(CloseHandle(h1).is_ok(), "failed to close first handle");
        }

        Ok(())
    }

    #[test]
    fn different_names_are_reported_as_new_mutexes() -> Result<()> {
        let name1 = unique_mutex_name();
        let name2 = unique_mutex_name();

        let (h1, created1) = create_named_mutex(&name1)?;
        let (h2, created2) = create_named_mutex(&name2)?;

        assert!(!h1.is_invalid(), "first handle should be valid");
        assert!(!h2.is_invalid(), "second handle should be valid");
        assert!(created1, "first mutex name should be newly created");
        assert!(created2, "second mutex name should be newly created");

        unsafe {
            assert!(CloseHandle(h2).is_ok(), "failed to close second handle");
            assert!(CloseHandle(h1).is_ok(), "failed to close first handle");
        }

        Ok(())
    }
}