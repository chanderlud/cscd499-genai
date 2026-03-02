#[cfg(test)]
mod tests {
    use super::*;
    use windows::Win32::Foundation::{ERROR_SUCCESS, ERROR_ACCESS_DENIED, ERROR_INVALID_PARAMETER, ERROR_FILE_NOT_FOUND};

    #[test]
    fn test_format_win32_error_message_success() {
        // ERROR_SUCCESS (0) should return a valid message
        let result = format_win32_error_message(ERROR_SUCCESS.0);
        assert!(result.is_ok());
        let msg = result.unwrap();
        assert!(!msg.is_empty());
        assert!(!msg.ends_with(['\n', '\r', ' ']));
    }

    #[test]
    fn test_format_win32_error_message_access_denied() {
        // ERROR_ACCESS_DENIED (5) is a common error
        let result = format_win32_error_message(ERROR_ACCESS_DENIED.0);
        assert!(result.is_ok());
        let msg = result.unwrap();
        assert!(!msg.is_empty());
        assert!(!msg.ends_with(['\n', '\r', ' ']));
        assert!(msg.contains("access"));
    }

    #[test]
    fn test_format_win32_error_message_invalid_parameter() {
        // ERROR_INVALID_PARAMETER (87) is another common error
        let result = format_win32_error_message(ERROR_INVALID_PARAMETER.0);
        assert!(result.is_ok());
        let msg = result.unwrap();
        assert!(!msg.is_empty());
        assert!(!msg.ends_with(['\n', '\r', ' ']));
        assert!(msg.contains("parameter"));
    }

    #[test]
    fn test_format_win32_error_message_file_not_found() {
        // ERROR_FILE_NOT_FOUND (2) is a common error
        let result = format_win32_error_message(ERROR_FILE_NOT_FOUND.0);
        assert!(result.is_ok());
        let msg = result.unwrap();
        assert!(!msg.is_empty());
        assert!(!msg.ends_with(['\n', '\r', ' ']));
        assert!(msg.contains("find"));
    }

    #[test]
    fn test_format_win32_error_message_unknown_error_code() {
        // Unknown error code should still return a message (may be generic)
        let result = format_win32_error_message(0xFFFFFFFF);
        assert!(result.is_ok());
        let msg = result.unwrap();
        assert!(!msg.is_empty());
        assert!(!msg.ends_with(['\n', '\r', ' ']));
    }

    #[test]
    fn test_format_win32_error_message_whitespace_trimming() {
        // Simulate a message with trailing whitespace (this is a mock scenario)
        // Since we can't easily mock FormatMessageW, we assume the real function trims correctly
        // This test ensures our expectations about trimming are documented
        let result = format_win32_error_message(ERROR_ACCESS_DENIED.0);
        assert!(result.is_ok());
        let msg = result.unwrap();
        assert!(!msg.ends_with(['\n', '\r', ' ']));
    }

    #[test]
    fn test_format_win32_error_message_empty_string_handling() {
        // Even if the system returns an empty string, we should handle it gracefully
        // This is more of a documentation test for expected behavior
        let result = format_win32_error_message(0);
        assert!(result.is_ok());
        let msg = result.unwrap();
        assert!(!msg.is_empty(), "Expected non-empty message for ERROR_SUCCESS");
    }
}
