#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_named_mutex_already_exists() {
        let name = "Local\\rust_windows_problem_mutex_example";

        // First call should return false (no existing mutex)
        let first = named_mutex_already_exists(name).unwrap();
        assert_eq!(first, false);

        // Second call should return true (mutex already exists)
        let second = named_mutex_already_exists(name).unwrap();
        assert_eq!(second, true);

        // Clean up: close the handle from the second call
        // Note: In real implementation, the handle is closed inside the function
    }

    #[test]
    fn test_named_mutex_already_exists_empty_name() {
        let result = named_mutex_already_exists("");
        assert!(result.is_err());
    }

    #[test]
    fn test_named_mutex_already_exists_invalid_name() {
        let name = "Local\\rust_windows_problem_mutex_example_invalid";
        let first = named_mutex_already_exists(name).unwrap();
        assert_eq!(first, false);

        // Try with a name that's too long or contains invalid characters
        let long_name = "Local\\".to_string() + &"a".repeat(1000);
        let result = named_mutex_already_exists(&long_name);
        assert!(result.is_err());
    }

    #[test]
    fn test_named_mutex_already_exists_multiple_instances() {
        let name = "Local\\rust_windows_problem_mutex_example_multiple";

        // First instance
        let first = named_mutex_already_exists(name).unwrap();
        assert_eq!(first, false);

        // Second instance
        let second = named_mutex_already_exists(name).unwrap();
        assert_eq!(second, true);

        // Third instance
        let third = named_mutex_already_exists(name).unwrap();
        assert_eq!(third, true);
    }

    #[test]
    fn test_named_mutex_already_exists_different_names() {
        let name1 = "Local\\rust_windows_problem_mutex_example_1";
        let name2 = "Local\\rust_windows_problem_mutex_example_2";

        // First instance of name1
        let first1 = named_mutex_already_exists(name1).unwrap();
        assert_eq!(first1, false);

        // First instance of name2 (should be independent)
        let first2 = named_mutex_already_exists(name2).unwrap();
        assert_eq!(first2, false);

        // Second instance of name1
        let second1 = named_mutex_already_exists(name1).unwrap();
        assert_eq!(second1, true);

        // Second instance of name2
        let second2 = named_mutex_already_exists(name2).unwrap();
        assert_eq!(second2, true);
    }
}