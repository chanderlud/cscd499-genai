#[cfg(test)]
mod tests {
    use super::set_file_hidden;
    use std::fs::{self, File};
    use std::os::windows::fs::MetadataExt;
    use std::path::PathBuf;
    use std::time::{SystemTime, UNIX_EPOCH};
    use windows::Win32::Storage::FileSystem::FILE_ATTRIBUTE_HIDDEN;

    fn unique_temp_file() -> PathBuf {
        let mut path = std::env::temp_dir();
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        path.push(format!("set_file_hidden_{nanos}.txt"));
        path
    }

    #[test]
    fn marks_file_as_hidden() {
        let path = unique_temp_file();
        File::create(&path).unwrap();

        set_file_hidden(&path).unwrap();

        let attrs = fs::metadata(&path).unwrap().file_attributes();
        assert_ne!(attrs & FILE_ATTRIBUTE_HIDDEN.0, 0);

        fs::remove_file(&path).unwrap();
    }
}
