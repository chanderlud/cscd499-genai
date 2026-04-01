use windows::core::{Error, Result};
use windows::Win32::Foundation::{HANDLE, WIN32_ERROR};
use windows::Win32::System::Memory::{CreateFileMappingA, PAGE_PROTECTION_FLAGS};

fn call_create_file_mapping_a() -> Result<HANDLE> {
    // Create a null handle for the file
    let hfile = HANDLE::default();

    // Call CreateFileMappingA with concrete parameters
    // CreateFileMappingA is unsafe and requires an unsafe block
    unsafe {
        CreateFileMappingA(
            hfile,
            None,
            PAGE_PROTECTION_FLAGS(0x40), // PAGE_READWRITE
            0,
            0,
            None,
        )
    }
}
