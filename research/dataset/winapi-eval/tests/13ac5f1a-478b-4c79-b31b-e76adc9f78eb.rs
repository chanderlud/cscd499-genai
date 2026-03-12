#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;
    use windows::Win32::UI::Shell::{
        FOLDERID_Desktop, FOLDERID_LocalAppData, FOLDERID_ProgramFiles,
    };

    #[test]
    fn test_known_folder_local_app_data_exists() {
        let id = FOLDERID_LocalAppData;
        let path = known_folder(id).unwrap();
        assert!(path.is_absolute(), "LocalAppData path should be absolute");
        assert!(path.exists(), "LocalAppData should exist");
    }

    #[test]
    fn test_known_folder_program_files_exists() {
        let id = FOLDERID_ProgramFiles;
        let path = known_folder(id).unwrap();
        assert!(path.is_absolute(), "ProgramFiles path should be absolute");
        assert!(path.exists(), "ProgramFiles should exist");
    }

    #[test]
    fn test_known_folder_desktop_exists() {
        let id = FOLDERID_Desktop;
        let path = known_folder(id).unwrap();
        assert!(path.is_absolute(), "Desktop path should be absolute");
        assert!(path.exists(), "Desktop should exist");
    }

    #[test]
    fn test_known_folder_invalid_guid_returns_error() {
        let invalid_guid = windows::core::GUID::from_values(0, 0, 0, [0; 8]);
        let result = known_folder(invalid_guid);
        assert!(result.is_err(), "Invalid GUID should return error");
    }

    #[test]
    fn test_known_folder_path_format_windows_style() {
        let id = FOLDERID_LocalAppData;
        let path = known_folder(id).unwrap();
        let path_str = path.to_string_lossy();
        assert!(
            path_str.contains(':'),
            "Path should be Windows format with drive letter"
        );
    }

    #[test]
    fn test_known_folder_path_not_empty() {
        let id = FOLDERID_LocalAppData;
        let path = known_folder(id).unwrap();
        assert!(!path.as_os_str().is_empty(), "Path should not be empty");
    }

    #[test]
    fn test_known_folder_path_is_valid_utf8() {
        let id = FOLDERID_LocalAppData;
        let path = known_folder(id).unwrap();
        assert!(
            path.to_string_lossy().is_utf8(),
            "Path should be valid UTF-8"
        );
    }
}
