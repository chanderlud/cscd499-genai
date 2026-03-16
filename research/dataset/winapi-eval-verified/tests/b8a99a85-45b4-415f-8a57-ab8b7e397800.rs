

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;
    use windows::Win32::System::Registry::{
        KEY_WRITE, REG_CREATE_KEY_DISPOSITION, REG_OPTION_NON_VOLATILE, RegCreateKeyExW,
        RegDeleteTreeW,
    };
    use std::ffi::OsStr;
    use std::iter::once;

    fn wide_null_test(s: &str) -> Vec<u16> {
        OsStr::new(s).encode_wide().chain(once(0)).collect()
    }

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
        let test_path = r"Software\RustRegListValuesTest\EmptyKey";
        let wide_path = wide_null_test(test_path);

        let mut hkey = HKEY::default();
        let mut disposition = REG_CREATE_KEY_DISPOSITION(0);

        let create_result = unsafe {
            RegCreateKeyExW(
                HKEY_CURRENT_USER,
                PCWSTR(wide_path.as_ptr()),
                Some(0),
                None,
                REG_OPTION_NON_VOLATILE,
                KEY_READ | KEY_WRITE,
                None,
                &mut hkey,
                Some(&mut disposition),
            )
        };

        assert_eq!(
            create_result, ERROR_SUCCESS,
            "Failed to create test registry key"
        );

        unsafe {
            let _ = RegCloseKey(hkey);
        };

        let names = reg_list_values_hkcu(test_path).unwrap();
        assert!(names.is_empty(), "Expected no values under empty test key");

        let cleanup_path = wide_null_test(r"Software\RustRegListValuesTest");
        unsafe {
            let _ = RegDeleteTreeW(HKEY_CURRENT_USER, PCWSTR(cleanup_path.as_ptr()));
        }
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
    fn test_reg_list_values_hkcu_path_with_backslash() {
        // Test path ending with backslash
        let result = reg_list_values_hkcu(r"Software\Microsoft\Windows\CurrentVersion\Run\");
        assert!(
            result.is_ok(),
            "Path with trailing backslash should be handled"
        );
    }
}