use windows::core::{Error, Result};
use windows::Win32::Foundation::HMODULE;
use windows::Win32::System::LibraryLoader::GetModuleHandleW;

fn call_get_module_handle_w() -> Result<HMODULE> {
    // Passing None retrieves the handle to the file used to create the calling process.
    // The windows crate wraps the Win32 API to return a Result, handling errors automatically.
    unsafe { GetModuleHandleW(None) }
}
