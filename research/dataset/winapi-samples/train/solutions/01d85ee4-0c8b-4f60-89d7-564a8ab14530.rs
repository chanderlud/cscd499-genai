use windows::core::{Error, Result, PCSTR, PCWSTR};
use windows::Win32::System::LibraryLoader::{GetModuleHandleW, GetProcAddress};

fn get_tick_count_addr() -> Result<usize> {
    // Convert module name to wide string for GetModuleHandleW
    let module_name: Vec<u16> = "kernel32.dll"
        .encode_utf16()
        .chain(std::iter::once(0))
        .collect();

    // Get module handle - unsafe block minimized to just the FFI call
    let module = unsafe { GetModuleHandleW(PCWSTR(module_name.as_ptr())) }?;

    // Convert function name to C string for GetProcAddress
    let func_name = std::ffi::CString::new("GetTickCount").map_err(|_| Error::from_thread())?;

    // Get function address - unsafe block minimized to just the FFI call
    let addr = unsafe { GetProcAddress(module, PCSTR(func_name.as_ptr() as *const u8)) };

    // Convert Option to Result, using GetLastError for error info
    addr.ok_or_else(Error::from_thread)
        .map(|proc| proc as usize)
}
