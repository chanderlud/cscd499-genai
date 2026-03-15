// Get current module base address using Win32 APIs

use windows::core::{Result, PCWSTR};
use windows::Win32::System::LibraryLoader::GetModuleHandleW;

fn get_base_address() -> Result<usize> {
    unsafe {
        // GetModuleHandleW with null returns handle to current executable
        let hmodule = GetModuleHandleW(PCWSTR::null())?;
        // HMODULE is the base address of the module
        Ok(hmodule.0 as usize)
    }
}

fn main() {
    match get_base_address() {
        Ok(base_addr) => println!("Base address: 0x{:x}", base_addr),
        Err(e) => eprintln!("Failed to get base address: {}", e),
    }
}
