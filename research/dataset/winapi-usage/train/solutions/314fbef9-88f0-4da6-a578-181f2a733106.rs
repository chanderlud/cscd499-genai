use windows::core::{Error, Result};
use windows::Win32::Foundation::HANDLE;
use windows::Win32::System::LibraryLoader::BeginUpdateResourceA;

fn call_begin_update_resource_a() -> Result<HANDLE> {
    // SAFETY: The string literal is null-terminated and valid for the duration of the call.
    unsafe { BeginUpdateResourceA(windows::core::s!("test.exe"), false) }
}
