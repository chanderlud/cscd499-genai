use std::ffi::OsStr;
use std::io;
use std::iter::once;
use std::os::windows::ffi::OsStrExt;
use windows::core::{Error, Result, HRESULT, PCSTR, PCWSTR};
use windows::Win32::Foundation::{FreeLibrary, HMODULE};
use windows::Win32::System::LibraryLoader::{GetProcAddress, LoadLibraryW};

fn wide_null(s: &OsStr) -> Vec<u16> {
    s.encode_wide().chain(once(0)).collect()
}

pub fn dynamic_get_current_process_id() -> io::Result<u32> {
    // Load kernel32.dll
    let module_name = wide_null(OsStr::new("kernel32.dll"));
    let module = unsafe { LoadLibraryW(PCWSTR(module_name.as_ptr())) };
    
    let module = match module {
        Ok(h) => h,
        Err(e) => {
            return Err(io::Error::new(
                io::ErrorKind::Other,
                format!("LoadLibraryW failed: {}", e),
            ));
        }
    };
    
    // Ensure we free the library even if GetProcAddress fails
    let result = (|| {
        // Get address of GetCurrentProcessId
        let proc_name = b"GetCurrentProcessId\0";
        let proc_addr = unsafe { GetProcAddress(module, PCSTR(proc_name.as_ptr())) };
        
        let proc_addr = match proc_addr {
            Some(addr) => addr,
            None => {
                let err = Error::from_win32();
                return Err(io::Error::new(
                    io::ErrorKind::Other,
                    format!("GetProcAddress failed: {}", err),
                ));
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