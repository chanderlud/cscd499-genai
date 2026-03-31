use windows::core::{w, Error, Result, HRESULT};
use windows::Win32::System::LibraryLoader::AddDllDirectory;

fn call_add_dll_directory() -> HRESULT {
    // SAFETY: AddDllDirectory expects a valid null-terminated wide string.
    // The w! macro guarantees a valid PCWSTR, and the call is safe.
    let cookie = unsafe { AddDllDirectory(w!("C:\\Windows\\System32")) };
    if cookie.is_null() {
        Error::from_thread().code()
    } else {
        HRESULT(0)
    }
}
