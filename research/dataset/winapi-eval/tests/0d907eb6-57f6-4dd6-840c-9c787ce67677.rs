#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::io::Write;
    use sha2::Digest;

    #[test]
    fn test_shell_stream_sha256_empty_file() {
        let temp = tempfile::tempdir().unwrap();
        let empty_path = temp.path().join("empty.txt");
        std::fs::File::create(&empty_path).unwrap();
        let digest = shell_stream_sha256(&empty_path).unwrap();
        assert_eq!(digest, [0u8; 32]);
    }

    #[test]
    fn test_shell_stream_sha256_small_file() {
        let temp = tempfile::tempdir().unwrap();
        let small_path = temp.path().join("small.txt");
        let content = b"Hello, world!";
        fs::write(&small_path, content).unwrap();
        let digest = shell_stream_sha256(&small_path).unwrap();
        let expected = sha2::Sha256::digest(content);
        assert_eq!(digest, expected.into());
    }

    #[test]
    fn test_shell_stream_sha256_large_file() {
        let temp = tempfile::tempdir().unwrap();
        let large_path = temp.path().join("large.bin");
        let mut data = vec![0u8; 1024 * 1024];
        rand::thread_rng().fill(&mut data[..]);
        fs::write(&large_path, &data).unwrap();
        let digest = shell_stream_sha256(&large_path).unwrap();
        let expected = sha2::Sha256::digest(&data);
        assert_eq!(digest, expected.into());
    }

    #[test]
    fn test_shell_stream_sha256_nonexistent_file() {
        let temp = tempfile::tempdir().unwrap();
        let nonexist_path = temp.path().join("nonexistent.txt");
        let result = shell_stream_sha256(&nonexist_path);
        assert!(result.is_err());
    }

    #[test]
    fn test_shell_stream_sha256_directory_path() {
        let temp = tempfile::tempdir().unwrap();
        let dir_path = temp.path().join("some_dir");
        fs::create_dir(&dir_path).unwrap();
        let result = shell_stream_sha256(&dir_path);
        assert!(result.is_err());
    }
}