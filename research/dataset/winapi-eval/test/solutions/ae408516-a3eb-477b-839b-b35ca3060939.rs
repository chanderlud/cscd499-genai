use std::io;
use std::os::windows::ffi::OsStrExt;
use std::path::Path;
use windows::core::PCWSTR;
use windows::Win32::System::Environment::SetCurrentDirectoryW;

pub fn change_current_directory<P: AsRef<Path>>(path: P) -> io::Result<()> {
    let wide_path: Vec<u16> = path
        .as_ref()
        .as_os_str()
        .encode_wide()
        .chain(std::iter::once(0))
        .collect();

    // Safety:
    // - `wide_path` is null-terminated.
    // - The pointer remains valid for the duration of the call.
    let ok = unsafe { SetCurrentDirectoryW(PCWSTR(wide_path.as_ptr())) }.as_bool();

    if ok {
        Ok(())
    } else {
        Err(io::Error::last_os_error())
    }
}
