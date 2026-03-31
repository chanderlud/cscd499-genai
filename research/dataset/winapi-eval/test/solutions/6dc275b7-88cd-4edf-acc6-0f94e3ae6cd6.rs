use windows::core::{Result, PCWSTR};
use windows::Win32::Storage::FileSystem::{SetFileAttributesW, FILE_ATTRIBUTE_READONLY};

pub fn make_file_read_only(path: &str) -> Result<()> {
    let wide_path: Vec<u16> = path.encode_utf16().chain(std::iter::once(0)).collect();

    unsafe { SetFileAttributesW(PCWSTR(wide_path.as_ptr()), FILE_ATTRIBUTE_READONLY) }
}
