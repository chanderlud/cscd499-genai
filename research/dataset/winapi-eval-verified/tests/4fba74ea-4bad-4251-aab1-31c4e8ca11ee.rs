#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_win32_error_message_success() {
        let result = format_win32_error_message(0);
        assert!(result.is_ok());
        let msg = result.unwrap();
        assert!(!msg.is_empty());
        assert!(!msg.ends_with(['\n', '\r', ' ']));
    }

    #[test]
    fn test_format_win32_error_message_access_denied() {
        let result = format_win32_error_message(5);
        assert!(result.is_ok());
        let msg = result.unwrap();
        assert!(!msg.is_empty());
        assert!(!msg.ends_with(['\n', '\r', ' ']));
        assert!(msg.to_lowercase().contains("access"));
    }

    #[test]
    fn test_format_win32_error_message_invalid_parameter() {
        let result = format_win32_error_message(87);
        assert!(result.is_ok());
        let msg = result.unwrap();
        assert!(!msg.is_empty());
        assert!(!msg.ends_with(['\n', '\r', ' ']));
        assert!(msg.to_lowercase().contains("parameter"));
    }

    #[test]
    fn test_format_win32_error_message_file_not_found() {
        let result = format_win32_error_message(2);
        assert!(result.is_ok());
        let msg = result.unwrap();
        assert!(!msg.is_empty());
        assert!(!msg.ends_with(['\n', '\r', ' ']));
        assert!(msg.to_lowercase().contains("find"));
    }

    #[test]
    fn test_format_win32_error_message_unknown_error_code() {
        let result = format_win32_error_message(0xFFFFFFFF);
        assert!(result.is_ok());
        let msg = result.unwrap();
        assert!(!msg.is_empty());
        assert!(!msg.ends_with(['\n', '\r', ' ']));
    }

    #[test]
    fn test_format_win32_error_message_zero_error_code() {
        let result = format_win32_error_message(0);
        assert!(result.is_ok());
        let msg = result.unwrap();
        assert!(!msg.is_empty());
        assert!(!msg.ends_with(['\n', '\r', ' ']));
    }

    #[test]
    fn test_format_win32_error_message_whitespace_trimming() {
        let result = format_win32_error_message(5);
        assert!(result.is_ok());
        let msg = result.unwrap();
        assert!(!msg.ends_with(['\n', '\r', ' ']));
    }

    #[test]
    fn test_format_win32_error_message_edge_case_high_value() {
        let result = format_win32_error_message(u32::MAX);
        assert!(result.is_ok());
        let msg = result.unwrap();
        assert!(!msg.is_empty());
        assert!(!msg.ends_with(['\n', '\r', ' ']));
    }
}
