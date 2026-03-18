use std::os::windows::ffi::OsStrExt;
use std::path::Path;
use windows::core::{Error, Result, HRESULT, PCWSTR};
use windows::Win32::Foundation::ERROR_FILENAME_EXCED_RANGE;
use windows::Win32::Storage::FileSystem::CreateHardLinkW;

const MAX_PATH: usize = 260;

fn path_to_wide_fixed(path: &Path, buffer: &mut [u16; MAX_PATH]) -> Result<PCWSTR> {
    let mut iter = path.as_os_str().encode_wide();
    let mut i = 0;

    // Copy characters until we reach the end or buffer limit (leaving room for null terminator)
    while i < MAX_PATH - 1 {
        match iter.next() {
            Some(c) => {
                buffer[i] = c;
                i += 1;
            }
            None => break,
        }
    }

    // Check if we consumed all characters
    if iter.next().is_some() {
        return Err(Error::from_hresult(HRESULT::from_win32(
            ERROR_FILENAME_EXCED_RANGE.0,
        )));
    }

    // Null terminate
    buffer[i] = 0;

    // Safety: We've ensured the buffer is null-terminated and contains valid UTF-16
    Ok(PCWSTR(buffer.as_ptr()))
}

pub fn create_hard_link(link_path: &Path, existing_file: &Path) -> Result<()> {
    let mut link_buffer = [0u16; MAX_PATH];
    let mut existing_buffer = [0u16; MAX_PATH];

    let link_wide = path_to_wide_fixed(link_path, &mut link_buffer)?;
    let existing_wide = path_to_wide_fixed(existing_file, &mut existing_buffer)?;

    // Safety: We've properly converted both paths to null-terminated wide strings
    unsafe {
        CreateHardLinkW(link_wide, existing_wide, None)?;
    }

    Ok(())
}
