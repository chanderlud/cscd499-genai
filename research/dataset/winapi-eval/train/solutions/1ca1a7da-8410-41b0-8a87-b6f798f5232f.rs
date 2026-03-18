use std::io;
use windows::core::{s, w, PCSTR, PCWSTR};
use windows::Win32::Foundation::FreeLibrary;
use windows::Win32::System::LibraryLoader::{GetProcAddress, LoadLibraryW};

pub fn dynamic_get_current_process_id() -> io::Result<u32> {
    // Load kernel32.dll
    let module = unsafe { LoadLibraryW(w!("kernel32.dll")) };

    let module = match module {
        Ok(h) => h,
        Err(_) => {
            return Err(io::Error::last_os_error());
        }
    };

    // Ensure we free the library even if GetProcAddress fails
    let result = (|| {
        // Get address of GetCurrentProcessId
        let proc_addr = unsafe { GetProcAddress(module, s!("GetCurrentProcessId")) };

        let proc_addr = match proc_addr {
            Some(addr) => addr,
            None => {
                return Err(io::Error::last_os_error());
            }
        };

        // Cast to function pointer and call
        let func: unsafe extern "system" fn() -> u32 = unsafe { std::mem::transmute(proc_addr) };
        let pid = unsafe { func() };

        Ok(pid)
    })();

    // Free the library
    unsafe { FreeLibrary(module) };

    result
}
