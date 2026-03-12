#[cfg(test)]
mod tests {
    use super::*;
    use windows::Win32::System::Threading::GetCurrentProcessId;

    #[test]
    fn test_dynamic_get_current_process_id_success() {
        let pid = dynamic_get_current_process_id().unwrap();
        assert_ne!(pid, 0);
        assert_eq!(pid, unsafe { GetCurrentProcessId() });
    }

    #[test]
    fn test_dynamic_get_current_process_id_returns_valid_pid() {
        let pid = dynamic_get_current_process_id().unwrap();
        assert!(pid > 0);
    }

    #[test]
    fn test_dynamic_get_current_process_id_same_as_direct_call() {
        let dynamic_pid = dynamic_get_current_process_id().unwrap();
        let direct_pid = unsafe { GetCurrentProcessId() };
        assert_eq!(dynamic_pid, direct_pid);
    }
}

