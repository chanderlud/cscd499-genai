#[cfg(test)]
mod tests {
    use super::copy_file_winapi;
    use std::fs;
    use std::path::PathBuf;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn unique_path(name: &str) -> PathBuf {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();

        let mut path = std::env::temp_dir();
        path.push(format!("{}_{}_{}.txt", name, std::process::id(), nanos));
        path
    }

    #[test]
    fn copies_file_contents() {
        let src = unique_path("copy_file_winapi_src");
        let dst = unique_path("copy_file_winapi_dst");

        fs::write(&src, b"hello windows api").unwrap();

        copy_file_winapi(&src, &dst, true).unwrap();

        assert_eq!(fs::read(&dst).unwrap(), b"hello windows api");

        let _ = fs::remove_file(&src);
        let _ = fs::remove_file(&dst);
    }

    #[test]
    fn fails_when_destination_exists_and_flag_is_true() {
        let src = unique_path("copy_file_winapi_src_exists");
        let dst = unique_path("copy_file_winapi_dst_exists");

        fs::write(&src, b"source data").unwrap();
        fs::write(&dst, b"existing data").unwrap();

        let result = copy_file_winapi(&src, &dst, true);

        assert!(result.is_err());
        assert_eq!(fs::read(&dst).unwrap(), b"existing data");

        let _ = fs::remove_file(&src);
        let _ = fs::remove_file(&dst);
    }

    #[test]
    fn overwrites_when_destination_exists_and_flag_is_false() {
        let src = unique_path("copy_file_winapi_src_overwrite");
        let dst = unique_path("copy_file_winapi_dst_overwrite");

        fs::write(&src, b"new contents").unwrap();
        fs::write(&dst, b"old contents").unwrap();

        copy_file_winapi(&src, &dst, false).unwrap();

        assert_eq!(fs::read(&dst).unwrap(), b"new contents");

        let _ = fs::remove_file(&src);
        let _ = fs::remove_file(&dst);
    }
}
