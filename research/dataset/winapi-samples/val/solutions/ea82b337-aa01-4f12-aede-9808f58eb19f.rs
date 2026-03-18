use std::ffi::OsStr;
use std::os::windows::ffi::OsStrExt;
use std::path::Path;
use windows::core::{Error, Result, HRESULT, PCWSTR};
use windows::Win32::Foundation::{ERROR_DIRECTORY, ERROR_FILENAME_EXCED_RANGE, MAX_PATH};
use windows::Win32::Storage::FileSystem::{
    CreateHardLinkW, GetFileAttributesW, FILE_ATTRIBUTE_DIRECTORY, INVALID_FILE_ATTRIBUTES,
};

fn to_wide_fixed<'a>(s: &OsStr, buf: &'a mut [u16; (MAX_PATH as usize) + 1]) -> Result<&'a [u16]> {
    let mut len = 0;
    for (i, u) in s.encode_wide().enumerate() {
        if i >= MAX_PATH as usize {
            return Err(Error::from_hresult(HRESULT::from_win32(
                ERROR_FILENAME_EXCED_RANGE.0,
            )));
        }
        buf[i] = u;
        len = i + 1;
    }
    buf[len] = 0;
    Ok(&buf[..len + 1])
}

pub fn create_hard_link(link_path: &Path, existing_file: &Path) -> Result<()> {
    let mut link_buf = [0u16; (MAX_PATH as usize) + 1];
    let mut existing_buf = [0u16; (MAX_PATH as usize) + 1];

    let link_wide = to_wide_fixed(link_path.as_os_str(), &mut link_buf)?;
    let existing_wide = to_wide_fixed(existing_file.as_os_str(), &mut existing_buf)?;

    // Check if target is a directory
    let attrs = unsafe { GetFileAttributesW(PCWSTR(existing_wide.as_ptr())) };
    if attrs == INVALID_FILE_ATTRIBUTES {
        return Err(Error::from_thread());
    }
    if (attrs & FILE_ATTRIBUTE_DIRECTORY.0) != 0 {
        return Err(Error::from_hresult(HRESULT::from_win32(ERROR_DIRECTORY.0)));
    }

    // Create the hard link
    unsafe {
        CreateHardLinkW(
            PCWSTR(link_wide.as_ptr()),
            PCWSTR(existing_wide.as_ptr()),
            None,
        )
    }?;

    Ok(())
}
