#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;

    #[test]
    fn test_named_pipe_impersonated_sid_happy_path() {
        let sid = named_pipe_impersonated_sid(r"\\.\pipe\imp_test").unwrap();
        assert!(!sid.is_empty(), "SID string should not be empty");
        assert!(
            sid.starts_with("S-"),
            "SID string should start with 'S-' prefix"
        );
    }

    #[test]
    fn test_named_pipe_impersonated_sid_invalid_pipe_name() {
        let result = named_pipe_impersonated_sid(r"\\.\pipe\invalid$%^&*");
        assert!(result.is_err(), "Invalid pipe name should return error");
    }

    #[test]
    fn test_named_pipe_impersonated_sid_empty_name() {
        let result = named_pipe_impersonated_sid("");
        assert!(result.is_err(), "Empty pipe name should return error");
    }

    #[test]
    fn test_named_pipe_impersonated_sid_nonexistent_pipe() {
        let result = named_pipe_impersonated_sid(r"\\.\pipe\nonexistent_test_pipe");
        assert!(result.is_err(), "Nonexistent pipe should return error");
    }

    #[test]
    fn test_named_pipe_impersonated_sid_concurrent_access() {
        let handle = thread::spawn(|| named_pipe_impersonated_sid(r"\\.\pipe\imp_test").unwrap());
        let sid = handle.join().unwrap();
        assert!(!sid.is_empty(), "Concurrent impersonation should succeed");
        assert!(sid.starts_with("S-"), "Concurrent SID should be valid");
    }

    #[test]
    fn test_named_pipe_impersonated_sid_multiple_clients() {
        let mut handles = vec![];
        for i in 0..3 {
            handles.push(thread::spawn(move || {
                named_pipe_impersonated_sid(r"\\.\pipe\imp_test").unwrap()
            }));
        }

        let sids: Vec<String> = handles.into_iter().map(|h| h.join().unwrap()).collect();

        assert!(!sids.is_empty(), "Should get SIDs from multiple clients");
        assert!(
            sids.iter()
                .all(|sid| !sid.is_empty() && sid.starts_with("S-")),
            "All SIDs should be valid"
        );
    }
}
