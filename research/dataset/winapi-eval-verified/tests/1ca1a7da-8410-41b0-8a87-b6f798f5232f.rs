#[cfg(test)]
#[cfg(windows)]
mod tests {
    use super::*;
    use windows::Win32::System::Threading::GetCurrentProcessId;

    #[test]
    fn test_dynamic_get_current_process_id_matches_direct_call() {
        let pid = dynamic_get_current_process_id().unwrap();
        assert_ne!(pid, 0);
        assert_eq!(pid, unsafe { GetCurrentProcessId() });
    }

    #[test]
    fn test_dynamic_get_current_process_id_is_stable_across_calls() {
        let pid1 = dynamic_get_current_process_id().unwrap();
        let pid2 = dynamic_get_current_process_id().unwrap();
        let direct_pid = unsafe { GetCurrentProcessId() };

        assert_ne!(pid1, 0);
        assert_eq!(pid1, pid2);
        assert_eq!(pid1, direct_pid);
    }
}
