use windows::core::{Error, Result, PCSTR, PCWSTR};
use windows::Win32::System::LibraryLoader::{GetModuleHandleW, GetProcAddress};

fn get_message_box_addr() -> Result<usize> {
    let module_name: Vec<u16> = "user32.dll"
        .encode_utf16()
        .chain(std::iter::once(0))
        .collect();
    let module = unsafe { GetModuleHandleW(PCWSTR(module_name.as_ptr())) }?;
    let func_name = std::ffi::CString::new("MessageBoxA")
        .map_err(|_| Error::from_hresult(windows::core::HRESULT(0x80004005u32 as i32)))?;
    let addr = unsafe { GetProcAddress(module, PCSTR(func_name.as_ptr() as *const u8)) };
    addr.ok_or_else(Error::from_thread).map(|a| a as usize)
}

fn main() -> Result<()> {
    let addr = get_message_box_addr()?;
    println!("MessageBoxA address: {:p}", addr as *const ());
    Ok(())
}
