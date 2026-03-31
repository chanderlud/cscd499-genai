use windows::core::{Result, PCWSTR};
use windows::Win32::Storage::FileSystem::CreateDirectoryW;

pub fn create_directory(path: &str) -> Result<()> {
    let wide_path: Vec<u16> = path.encode_utf16().chain(std::iter::once(0)).collect();

    unsafe { CreateDirectoryW(PCWSTR(wide_path.as_ptr()), None) }
}
