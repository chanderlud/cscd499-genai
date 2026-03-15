//! Get current module's entry point address via PE header parsing

use windows::{
    core::Result,
    Win32::{
        Foundation::ERROR_INVALID_DATA,
        System::{
            Diagnostics::Debug::IMAGE_NT_HEADERS32,
            LibraryLoader::GetModuleHandleA,
            SystemServices::{IMAGE_DOS_HEADER, IMAGE_DOS_SIGNATURE, IMAGE_NT_SIGNATURE},
        },
    },
};

fn get_application_entry_addr() -> Result<usize> {
    // Get handle to current module
    let module = unsafe { GetModuleHandleA(None) }?;
    let module_addr = module.0 as usize;

    // Read and validate DOS header
    let dos_header = unsafe { (module_addr as *const IMAGE_DOS_HEADER).read() };
    if dos_header.e_magic != IMAGE_DOS_SIGNATURE {
        return Err(windows::core::Error::from_hresult(
            ERROR_INVALID_DATA.to_hresult(),
        ));
    }

    // Read and validate NT headers
    let nt_headers_ptr: *const IMAGE_NT_HEADERS32 =
        (module_addr + dos_header.e_lfanew as usize) as _;
    let nt_headers = unsafe { nt_headers_ptr.read() };
    if nt_headers.Signature != IMAGE_NT_SIGNATURE {
        return Err(windows::core::Error::from_hresult(
            ERROR_INVALID_DATA.to_hresult(),
        ));
    }

    // Calculate entry point address from RVA
    let entry_point_rva = nt_headers.OptionalHeader.AddressOfEntryPoint;
    let entry_point_addr = module_addr + entry_point_rva as usize;
    Ok(entry_point_addr)
}

fn main() -> Result<()> {
    let entry_point = get_application_entry_addr()?;
    println!("Application entry point: 0x{:X}", entry_point);
    Ok(())
}
