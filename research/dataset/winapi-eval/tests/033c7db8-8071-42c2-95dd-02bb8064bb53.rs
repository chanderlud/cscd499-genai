#[cfg(test)]
mod tests {
    use super::*;
    use std::io;

    #[test]
    fn test_named_mutex_already_exists_basic_flow() {
        let name = "Local\\rust_windows_problem_mutex_example_basic";
        let first = named_mutex_already_exists(name).unwrap();
        assert_eq!(first, false);
        let second = named_mutex_already_exists(name).unwrap();
        assert_eq!(second, true);
    }

    #[test]
    fn test_named_mutex_already_exists_empty_name() {
        let result = named_mutex_already_exists("");
        assert!(result.is_err());
    }

    #[test]
    fn test_named_mutex_already_exists_invalid_characters() {
        let name = "Local\\rust_windows_problem_mutex_example_invalid\x00";
        let result = named_mutex_already_exists(name);
        assert!(result.is_err());
    }

    #[test]
    fn test_named_mutex_already_exists_very_long_name() {
        let long_name = "Local\\".to_string() + &"a".repeat(1000);
        let result = named_mutex_already_exists(&long_name);
        assert!(result.is_err());
    }

    #[test]
    fn test_named_mutex_already_exists_multiple_instances() {
        let name = "Local\\rust_windows_problem_mutex_example_multiple";
        let first = named_mutex_already_exists(name).unwrap();
        assert_eq!(first, false);
        let second = named_mutex_already_exists(name).unwrap();
        assert_eq!(second, true);
        let third = named_mutex_already_exists(name).unwrap();
        assert_eq!(third, true);
    }

    #[test]
    fn test_named_mutex_already_exists_different_names() {
        let name1 = "Local\\rust_windows_problem_mutex_example_1";
        let name2 = "Local\\rust_windows_problem_mutex_example_2";

        let first1 = named_mutex_already_exists(name1).unwrap();
        assert_eq!(first1, false);
        let first2 = named_mutex_already_exists(name2).unwrap();
        assert_eq!(first2, false);

        let second1 = named_mutex_already_exists(name1).unwrap();
        assert_eq!(second1, true);
        let second2 = named_mutex_already_exists(name2).unwrap();
        assert_eq!(second2, true);
    }

    #[test]
    fn test_named_mutex_already_exists_cleanup() {
        let name = "Local\\rust_windows_problem_mutex_example_cleanup";
        let first = named_mutex_already_exists(name).unwrap();
        assert_eq!(first, false);
        let second = named_mutex_already_exists(name).unwrap();
        assert_eq!(second, true);
    }

    #[test]
    fn test_named_mutex_already_exists_case_sensitivity() {
        let name = "Local\\rust_windows_problem_mutex_example_case";
        let name_upper = "Local\\RUST_WINDOWS_PROBLEM_MUTEX_EXAMPLE_CASE";

        let first = named_mutex_already_exists(name).unwrap();
        assert_eq!(first, false);
        let first_upper = named_mutex_already_exists(name_upper).unwrap();
        assert_eq!(first_upper, false);
    }
}
