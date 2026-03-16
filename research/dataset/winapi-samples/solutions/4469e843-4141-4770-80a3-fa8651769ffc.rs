// TITLE: Get Instance Handle Example

use windows::core::{Error, Result};
use windows::Win32::Foundation::HMODULE;
use windows::Win32::System::SystemServices::IMAGE_DOS_HEADER;

/// Gets the instance handle by taking the address of the
/// pseudo-variable created by the microsoft linker:
/// https://devblogs.microsoft.com/oldnewthing/20041025-00/?p=37483
///
/// This is preferred over GetModuleHandle(NULL) because it also works in DLLs:
/// https://stackoverflow.com/questions/21718027/getmodulehandlenull-vs-hinstance
pub fn get_instance_handle() -> HMODULE {
    extern "C" {
        static __ImageBase: IMAGE_DOS_HEADER;
    }

    // SAFETY: Taking the address of the linker-provided __ImageBase symbol is safe
    // as it's guaranteed to exist and be valid for the lifetime of the module.
    unsafe { HMODULE(&__ImageBase as *const _ as *mut _) }
}

fn main() -> Result<()> {
    let instance = get_instance_handle();
    println!("Instance handle: {:?}", instance);
    Ok(())
}
