use std::path::Path;
use windows::Win32::Storage::FileSystem::CreateHardLinkW;
use windows::core::{PCWSTR, Result};

fn wide_null(s: &std::ffi::OsStr) -> Vec<u16> {
    use std::{iter::once, os::windows::ffi::OsStrExt};
    s.encode_wide().chain(once(0)).collect()
}

pub fn create_hard_link(link_path: &Path, existing_file: &Path) -> Result<()> {
    let link_wide = wide_null(link_path.as_os_str());
    let existing_wide = wide_null(existing_file.as_os_str());

    // SAFETY: We're passing valid null-terminated wide strings to CreateHardLinkW.
    // The function will fail if the target is a directory, as required.
    unsafe {
        CreateHardLinkW(
            PCWSTR(link_wide.as_ptr()),
            PCWSTR(existing_wide.as_ptr()),
            None,
        )
    }
}
