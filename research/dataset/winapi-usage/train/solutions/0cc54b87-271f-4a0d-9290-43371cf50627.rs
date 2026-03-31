use windows::core::{Error, Result};
use windows::Win32::Storage::FileSystem::CreateDirectoryW;

#[allow(dead_code)]
fn call_create_directory_w() -> Result<()> {
    let path = windows::core::w!("C:\\temp\\test_dir");
    // SAFETY: `path` is a valid null-terminated wide string pointer, and `None` correctly
    // represents a null pointer for the optional security attributes parameter.
    unsafe { CreateDirectoryW(path, None)? };
    Ok(())
}
