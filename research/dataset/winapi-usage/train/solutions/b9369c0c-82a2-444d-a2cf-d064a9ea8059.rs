use windows::core::{Error, Result};
use windows::Win32::Foundation::WIN32_ERROR;
use windows::Win32::System::LibraryLoader::BeginUpdateResourceA;

fn call_begin_update_resource_a() -> WIN32_ERROR {
    // SAFETY: Passing a static string literal and false is safe for BeginUpdateResourceA.
    match unsafe { BeginUpdateResourceA(windows::core::s!("test.exe"), false) } {
        Ok(_) => WIN32_ERROR(0),
        Err(e) => WIN32_ERROR(e.code().0 as u32),
    }
}
