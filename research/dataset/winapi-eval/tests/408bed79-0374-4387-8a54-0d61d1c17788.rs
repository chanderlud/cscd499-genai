#[cfg(test)]
mod tests {
    use super::*;
    use std::ffi::OsString;
    use std::path::Path;

    #[test]
    fn test_list_dir_nonexistent() {
        let path = Path::new(r"C:\nonexistent");
        assert!(list_dir(path).is_err());
    }

    #[test]
    fn test_list_dir_empty_directory() {
        let dir = tempfile::tempdir().unwrap();
        let names = list_dir(dir.path()).unwrap();
        assert!(names.is_empty());
    }

    #[test]
    fn test_list_dir_with_files_and_dirs() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(dir.path().join("file1.txt"), b"content").unwrap();
        std::fs::write(dir.path().join("file2.txt"), b"content").unwrap();
        std::fs::create_dir(dir.path().join("subdir")).unwrap();

        let mut names = list_dir(dir.path()).unwrap();
        names.sort();
        assert_eq!(
            names,
            vec![
                OsString::from("file1.txt"),
                OsString::from("file2.txt"),
                OsString::from("subdir")
            ]
        );
    }

    #[test]
    fn test_list_dir_excludes_dot_and_dotdot() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::create_dir(dir.path().join("subdir")).unwrap();

        let names = list_dir(dir.path()).unwrap();
        assert!(!names.iter().any(|name| name == "." || name == ".."));
    }

    #[test]
    fn test_list_dir_with_special_chars() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(dir.path().join("特殊字符.txt"), b"content").unwrap();
        std::fs::write(dir.path().join("file with spaces.txt"), b"content").unwrap();

        let mut names = list_dir(dir.path()).unwrap();
        names.sort();
        assert_eq!(
            names,
            vec![
                OsString::from("特殊字符.txt"),
                OsString::from("file with spaces.txt")
            ]
        );
    }
}
