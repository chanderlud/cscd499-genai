#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    #[test]
    fn test_final_path_from_handle_basic() {
        let path = Path::new("C:\\Windows\\System32\\notepad.exe");
        let result = final_path_from_handle(path);
        assert!(result.is_ok());
        let path_str = result.unwrap();
        assert!(path_str.to_lowercase().contains("notepad.exe"));
    }

    #[test]
    fn test_final_path_from_handle_relative_path() {
        let path = Path::new("C:\\Windows\\System32\\..\\notepad.exe");
        let result = final_path_from_handle(path);
        assert!(result.is_ok());
        let path_str = result.unwrap();
        assert!(path_str.to_lowercase().contains("notepad.exe"));
    }

    #[test]
    fn test_final_path_from_handle_long_path() {
        let path = Path::new("C:\\Windows\\System32\\drivers\\etc\\hosts");
        let result = final_path_from_handle(path);
        assert!(result.is_ok());
        let path_str = result.unwrap();
        assert!(path_str.to_lowercase().contains("hosts"));
    }

    #[test]
    fn test_final_path_from_handle_nonexistent_file() {
        let path = Path::new("C:\\nonexistent_file_123456789.txt");
        let result = final_path_from_handle(path);
        assert!(result.is_err());
    }

    #[test]
    fn test_final_path_from_handle_directory() {
        let path = Path::new("C:\\Windows\\System32");
        let result = final_path_from_handle(path);
        assert!(result.is_ok());
        let path_str = result.unwrap();
        assert!(path_str.to_lowercase().contains("system32"));
    }

    #[test]
    fn test_final_path_from_handle_unc_path() {
        let path = Path::new("\\\\?\\C:\\Windows\\System32\\notepad.exe");
        let result = final_path_from_handle(path);
        assert!(result.is_ok());
        let path_str = result.unwrap();
        assert!(path_str.to_lowercase().contains("notepad.exe"));
    }

    #[test]
    fn test_final_path_from_handle_empty_path() {
        let path = Path::new("");
        let result = final_path_from_handle(path);
        assert!(result.is_err());
    }

    #[test]
    fn test_final_path_from_handle_root() {
        let path = Path::new("C:\\");
        let result = final_path_from_handle(path);
        assert!(result.is_ok());
        let path_str = result.unwrap();
        let path = Path::new(&path_str);
        // Check that the path has a root and that the only component is the root.
        assert!(path.has_root());
        let mut components = path.components();
        // The first component is the root.
        assert!(components.next().is_some());
        // There should be no more components.
        assert!(components.next().is_none());
    }
}
