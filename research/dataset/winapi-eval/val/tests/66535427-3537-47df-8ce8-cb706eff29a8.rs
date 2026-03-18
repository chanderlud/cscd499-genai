#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    #[test]
    fn test_verify_authenticode_valid_system_file_success() {
        // Test a known valid system file that should be signed
        let path = Path::new(r"C:\Windows\System32\kernel32.dll");
        assert!(verify_authenticode(path).is_ok());
    }

    #[test]
    fn test_verify_authenticode_nonexistent_file_failure() {
        // Test that a nonexistent file returns an error
        let path = Path::new("nonexistent.exe");
        assert!(verify_authenticode(path).is_err());
    }

    #[test]
    fn test_verify_authenticode_empty_file_failure() {
        // Test that an empty file returns an error
        let temp_file = tempfile::Builder::new()
            .prefix("empty")
            .suffix(".exe")
            .tempfile()
            .unwrap();
        assert!(verify_authenticode(temp_file.path()).is_err());
    }

    #[test]
    fn test_verify_authenticode_unsupported_file_type_failure() {
        // Test that an unsupported file type (e.g., text file) returns an error
        let temp_file = tempfile::Builder::new()
            .prefix("text")
            .suffix(".txt")
            .tempfile()
            .unwrap();
        assert!(verify_authenticode(temp_file.path()).is_err());
    }

    #[test]
    fn test_verify_authenticode_corrupted_executable_failure() {
        // Test that a corrupted executable returns an error
        let temp_file = tempfile::Builder::new()
            .prefix("corrupted")
            .suffix(".exe")
            .tempfile()
            .unwrap();
        // Write some random data to simulate corruption
        let mut data = vec![0u8; 1024];
        for i in 0..data.len() {
            data[i] = (i % 256) as u8;
        }
        std::fs::write(temp_file.path(), &data).unwrap();
        assert!(verify_authenticode(temp_file.path()).is_err());
    }
}
