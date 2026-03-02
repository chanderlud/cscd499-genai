#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::path::PathBuf;

    fn temp_file_path() -> PathBuf {
        let mut path = std::env::temp_dir();
        path.push("append_test.txt");
        path
    }

    #[test]
    fn test_append_to_new_file() {
        let path = temp_file_path();
        let data = b"hello";
        append_all(&path, data).unwrap();
        let contents = fs::read(&path).unwrap();
        assert_eq!(contents, data);
        fs::remove_file(&path).unwrap();
    }

    #[test]
    fn test_append_to_existing_file() {
        let path = temp_file_path();
        let initial = b"start";
        let append_data = b"end";
        fs::write(&path, initial).unwrap();
        append_all(&path, append_data).unwrap();
        let contents = fs::read(&path).unwrap();
        assert_eq!(contents, b"startend");
        fs::remove_file(&path).unwrap();
    }

    #[test]
    fn test_append_empty_data() {
        let path = temp_file_path();
        let data = b"content";
        fs::write(&path, data).unwrap();
        append_all(&path, b"").unwrap();
        let contents = fs::read(&path).unwrap();
        assert_eq!(contents, data);
        fs::remove_file(&path).unwrap();
    }

    #[test]
    fn test_append_to_nonexistent_directory() {
        let mut path = temp_file_path();
        path.pop();
        path.push("nonexistent");
        path.push("file.txt");
        let data = b"data";
        let result = append_all(&path, data);
        assert!(result.is_err());
    }

    #[test]
    fn test_append_multiple_times() {
        let path = temp_file_path();
        let parts = [b"one", b"two", b"tre"];
        for part in parts {
            append_all(&path, part).unwrap();
        }
        let contents = fs::read(&path).unwrap();
        assert_eq!(contents, b"onetwothree");
        fs::remove_file(&path).unwrap();
    }
}