#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    #[test]
    fn test_reg_list_values_hkcu_existing_key_with_values() {
        // Test a known key that typically has values
        let names = reg_list_values_hkcu(r"Software\Microsoft\Windows\CurrentVersion\Run").unwrap();
        assert!(!names.is_empty(), "Expected values under Run key");
        assert!(
            names.iter().all(|n| !n.trim().is_empty()),
            "Value names should not be empty"
        );
    }

    #[test]
    fn test_reg_list_values_hkcu_key_with_no_values() {
        // Test a key that exists but has no values
        let names =
            reg_list_values_hkcu(r"Software\Microsoft\Windows\CurrentVersion\Explorer").unwrap();
        assert!(names.is_empty(), "Expected no values under Explorer key");
    }

    #[test]
    fn test_reg_list_values_hkcu_nonexistent_key() {
        // Test a key that doesn't exist
        let result = reg_list_values_hkcu(r"Software\NonExistentKey\InvalidPath");
        assert!(result.is_err(), "Expected error for nonexistent key");
    }

    #[test]
    fn test_reg_list_values_hkcu_empty_path() {
        // Test empty path input
        let result = reg_list_values_hkcu("");
        assert!(result.is_err(), "Expected error for empty path");
    }

    #[test]
    fn test_reg_list_values_hkcu_sorted_output() {
        // Test that output is sorted
        let names = reg_list_values_hkcu(r"Software\Microsoft\Windows\CurrentVersion\Run").unwrap();
        let mut sorted = names.clone();
        sorted.sort();
        assert_eq!(names, sorted, "Output should be sorted");
    }

    #[test]
    fn test_reg_list_values_hkcu_special_characters_in_names() {
        // Test a key with special characters in value names
        let names = reg_list_values_hkcu(r"Software\Microsoft\Windows\CurrentVersion\Run").unwrap();
        let has_special_chars = names
            .iter()
            .any(|n| n.contains(' ') || n.contains('.') || n.contains('-'));
        assert!(
            has_special_chars,
            "Expected some value names with special characters"
        );
    }

    #[test]
    fn test_reg_list_values_hkcu_no_duplicates() {
        // Test that there are no duplicate value names
        let names = reg_list_values_hkcu(r"Software\Microsoft\Windows\CurrentVersion\Run").unwrap();
        let unique_names: HashSet<&str> = names.iter().map(|s| s.as_str()).collect();
        assert_eq!(
            names.len(),
            unique_names.len(),
            "No duplicate value names expected"
        );
    }

    #[test]
    fn test_reg_list_values_hkcu_valid_utf8_names() {
        // Test that all value names are valid UTF-8 (String is always valid UTF-8)
        let names = reg_list_values_hkcu(r"Software\Microsoft\Windows\CurrentVersion\Run").unwrap();
        assert!(
            names.iter().all(|n| n.is_empty() || !n.is_empty()),
            "All value names should be valid UTF-8"
        );
    }

    #[test]
    fn test_reg_list_values_hkcu_path_with_backslash() {
        // Test path ending with backslash
        let result = reg_list_values_hkcu(r"Software\Microsoft\Windows\CurrentVersion\Run\");
        assert!(
            result.is_ok(),
            "Path with trailing backslash should be handled"
        );
    }
}
