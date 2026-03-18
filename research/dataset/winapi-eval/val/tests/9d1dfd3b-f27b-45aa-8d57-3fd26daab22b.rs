#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::io::Write;

    #[test]
    fn test_create_hard_link_success() {
        let temp_dir = tempfile::tempdir().unwrap();
        let existing_file = temp_dir.path().join("existing.txt");
        let mut file = fs::File::create(&existing_file).unwrap();
        file.write_all(b"test content").unwrap();
        let link_path = temp_dir.path().join("link.txt");

        create_hard_link(&link_path, &existing_file).unwrap();

        assert!(link_path.is_file(), "link_path should be a file");
        assert_eq!(
            fs::read_to_string(&link_path).unwrap(),
            "test content",
            "link should contain the same content as the original file"
        );

        // Verify both paths refer to the same underlying file by mutating one
        // and observing the change through the other.
        fs::write(&link_path, b"updated through link").unwrap();
        assert_eq!(
            fs::read_to_string(&existing_file).unwrap(),
            "updated through link",
            "writing through the hard link should update the original file"
        );
    }

    #[test]
    fn test_create_hard_link_to_directory_fails() {
        let temp_dir = tempfile::tempdir().unwrap();
        let existing_dir = temp_dir.path().join("dir");
        fs::create_dir(&existing_dir).unwrap();
        let link_path = temp_dir.path().join("link.txt");

        let result = create_hard_link(&link_path, &existing_dir);

        assert!(
            result.is_err(),
            "creating hard link to directory should fail"
        );
    }

    #[test]
    fn test_create_hard_link_nonexistent_target_fails() {
        let temp_dir = tempfile::tempdir().unwrap();
        let existing_file = temp_dir.path().join("missing.txt");
        let link_path = temp_dir.path().join("link.txt");

        let result = create_hard_link(&link_path, &existing_file);

        assert!(
            result.is_err(),
            "creating hard link to nonexistent file should fail"
        );
    }

    #[test]
    fn test_create_hard_link_to_different_file_success() {
        let temp_dir = tempfile::tempdir().unwrap();
        let existing_file = temp_dir.path().join("existing.txt");
        fs::File::create(&existing_file).unwrap();
        let link_path = temp_dir.path().join("link.txt");

        create_hard_link(&link_path, &existing_file).unwrap();

        assert!(link_path.is_file(), "link_path should be a file");
        assert!(
            existing_file.is_file(),
            "existing_file should still be a file"
        );

        fs::write(&existing_file, b"written to original").unwrap();
        assert_eq!(
            fs::read_to_string(&link_path).unwrap(),
            "written to original",
            "writing to the original file should be visible through the hard link"
        );
    }

    #[test]
    fn test_create_hard_link_existing_target_fails() {
        let temp_dir = tempfile::tempdir().unwrap();
        let existing_file = temp_dir.path().join("existing.txt");
        fs::File::create(&existing_file).unwrap();
        let link_path = temp_dir.path().join("link.txt");
        fs::File::create(&link_path).unwrap();

        let result = create_hard_link(&link_path, &existing_file);

        assert!(
            result.is_err(),
            "creating hard link when link_path already exists should fail"
        );
    }

    #[test]
    fn test_create_hard_link_multiple_links_same_target() {
        let temp_dir = tempfile::tempdir().unwrap();
        let existing_file = temp_dir.path().join("existing.txt");
        let mut file = fs::File::create(&existing_file).unwrap();
        file.write_all(b"test content").unwrap();

        let link_path1 = temp_dir.path().join("link1.txt");
        let link_path2 = temp_dir.path().join("link2.txt");

        create_hard_link(&link_path1, &existing_file).unwrap();
        create_hard_link(&link_path2, &existing_file).unwrap();

        assert!(link_path1.is_file(), "link_path1 should be a file");
        assert!(link_path2.is_file(), "link_path2 should be a file");
        assert_eq!(
            fs::read_to_string(&link_path1).unwrap(),
            "test content",
            "link1 should contain the same content"
        );
        assert_eq!(
            fs::read_to_string(&link_path2).unwrap(),
            "test content",
            "link2 should contain the same content"
        );

        // Mutate one link and verify the change is visible everywhere.
        fs::write(&link_path1, b"shared content").unwrap();

        assert_eq!(
            fs::read_to_string(&existing_file).unwrap(),
            "shared content",
            "existing file should reflect writes through link1"
        );
        assert_eq!(
            fs::read_to_string(&link_path2).unwrap(),
            "shared content",
            "link2 should reflect writes through link1"
        );
    }

    #[test]
    fn test_create_hard_link_with_relative_paths() {
        let current_dir = std::env::current_dir().unwrap();
        let temp_dir = tempfile::tempdir_in(&current_dir).unwrap();

        let existing_file = temp_dir.path().join("existing.txt");
        fs::File::create(&existing_file).unwrap();
        let link_path = temp_dir.path().join("link.txt");

        let existing_rel = existing_file.strip_prefix(&current_dir).unwrap();
        let link_rel = link_path.strip_prefix(&current_dir).unwrap();

        create_hard_link(link_rel, existing_rel).unwrap();

        assert!(
            link_path.is_file(),
            "link should be created with relative paths"
        );

        fs::write(&link_path, b"relative path content").unwrap();
        assert_eq!(
            fs::read_to_string(&existing_file).unwrap(),
            "relative path content",
            "relative paths should still produce a working hard link"
        );
    }
}
