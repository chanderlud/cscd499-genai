use windows::core::{Error, Result};
use windows::Win32::System::LibraryLoader::GetModuleFileNameW;

fn call_get_module_file_name_w() -> windows::core::Result<u32> {
    // Create a buffer for the filename (MAX_PATH is 260)
    let mut buffer = [0u16; 260];

    // Call GetModuleFileNameW with NULL for hmodule (current module)
    // This is unsafe because the API is marked as unsafe
    let result = unsafe { GetModuleFileNameW(None, &mut buffer) };

    // Check for error (returns 0 on error)
    if result == 0 {
        return Err(Error::from_thread());
    }

    Ok(result)
}
