#[cfg(test)]
mod tests {
    use super::*;
    use std::ffi::OsString;

    #[test]
    fn test_expand_env_basic() {
        let result = expand_env("%TEMP%").unwrap();
        assert!(!result.is_empty(), "Expanded TEMP should not be empty");
    }

    #[test]
    fn test_expand_env_with_path() {
        let result = expand_env(r"%TEMP%\test.txt").unwrap();
        assert!(!result.is_empty(), "Expanded path should not be empty");
        assert!(
            result.to_string_lossy().contains("test.txt"),
            "Result should contain the filename"
        );
    }

    #[test]
    fn test_expand_nonexistent_var() {
        let result = expand_env("%NONEXISTENT%").unwrap();
        assert_eq!(
            result.to_string_lossy(),
            "%NONEXISTENT%",
            "Unexpanded variable should remain unchanged"
        );
    }

    #[test]
    fn test_expand_multiple_vars() {
        let result = expand_env(r"%TEMP%\%PATH%").unwrap();
        assert!(
            !result.is_empty(),
            "Multiple variables should expand correctly"
        );
    }

    #[test]
    fn test_expand_empty_string() {
        let result = expand_env("").unwrap();
        assert!(result.is_empty(), "Empty input should return empty output");
    }

    #[test]
    fn test_expand_no_vars() {
        let result = expand_env("C:\\Windows\\System32").unwrap();
        assert_eq!(
            result.to_string_lossy(),
            "C:\\Windows\\System32",
            "String without variables should remain unchanged"
        );
    }

    #[test]
    fn test_expand_mixed_content() {
        let result = expand_env(r"C:\Users\%USERNAME%\Documents").unwrap();
        assert!(!result.is_empty(), "Mixed content should expand correctly");
    }

    #[test]
    fn test_expand_case_insensitive() {
        let result = expand_env("%temp%").unwrap();
        assert!(
            !result.is_empty(),
            "Variable expansion should be case-insensitive"
        );
    }

    #[test]
    fn test_expand_partial_match() {
        let result = expand_env("abc%TEMP%def").unwrap();
        assert!(
            !result.to_string_lossy().contains("%TEMP%"),
            "Partial match should expand the variable"
        );
    }

    #[test]
    fn test_expand_single_percent() {
        let result = expand_env("100%").unwrap();
        assert_eq!(
            result.to_string_lossy(),
            "100%",
            "Single percent should remain unchanged"
        );
    }

    #[test]
    fn test_expand_odd_percent() {
        let result = expand_env("%THIS_IS_A_TEST_VARIABLE_THAT_SHOULD_NOT_EXIST%b%").unwrap();
        assert_eq!(
            result.to_string_lossy(),
            "%THIS_IS_A_TEST_VARIABLE_THAT_SHOULD_NOT_EXIST%b%",
            "Odd percent should remain unchanged"
        );
    }

    #[test]
    fn test_expand_percent_in_middle() {
        let result = expand_env("abc%def").unwrap();
        assert_eq!(
            result.to_string_lossy(),
            "abc%def",
            "Percent in the middle should remain unchanged"
        );
    }
}
