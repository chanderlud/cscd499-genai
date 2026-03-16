// TITLE: Get module file name using GetModuleFileNameW

use windows::core::{Error, Result, HRESULT};
use windows::Win32::Foundation::{HINSTANCE, HMODULE, MAX_PATH};
use windows::Win32::System::LibraryLoader::GetModuleFileNameW;

fn get_module_path(instance: HINSTANCE) -> Result<String> {
    let mut path = [0u16; MAX_PATH as usize];
    // SAFETY: GetModuleFileNameW writes to the provided buffer and returns the length
    let path_len = unsafe { GetModuleFileNameW(Some(HMODULE(instance.0)), &mut path) } as usize;
    if path_len == 0 {
        // Failed to get module file name
        return Err(Error::from_thread());
    }
    String::from_utf16(&path[0..path_len]).map_err(|_| {
        // UTF-16 conversion failed
        Error::from_hresult(HRESULT::from_win32(0x80004005)) // E_FAIL equivalent
    })
}

fn main() -> Result<()> {
    // Get the path of the current executable module
    let module_path = get_module_path(HINSTANCE::default())?;
    println!("Module path: {}", module_path);
    Ok(())
}
