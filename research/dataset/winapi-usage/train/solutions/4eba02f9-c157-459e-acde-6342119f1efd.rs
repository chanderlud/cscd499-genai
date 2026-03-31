use windows::core::{w, Error, Result};
use windows::Win32::Foundation::WIN32_ERROR;
use windows::Win32::System::LibraryLoader::AddDllDirectory;

fn call_add_dll_directory() -> WIN32_ERROR {
    let ptr = unsafe { AddDllDirectory(w!("C:\\")) };
    if ptr.is_null() {
        WIN32_ERROR(Error::from_thread().code().0 as u32)
    } else {
        WIN32_ERROR(0)
    }
}
