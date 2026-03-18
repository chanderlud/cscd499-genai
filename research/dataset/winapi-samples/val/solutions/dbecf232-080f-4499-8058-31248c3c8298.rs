#![allow(non_upper_case_globals)]

use std::collections::HashMap;
use std::ffi::CStr;

use windows::core::{Error, Result, HRESULT};
use windows::Win32::Foundation::ERROR_INVALID_DATA;
use windows::Win32::System::Diagnostics::Debug::{
    IMAGE_DIRECTORY_ENTRY_IMPORT, IMAGE_NT_HEADERS32,
};
use windows::Win32::System::LibraryLoader::GetModuleHandleA;
use windows::Win32::System::SystemServices::{
    IMAGE_DOS_HEADER, IMAGE_DOS_SIGNATURE, IMAGE_IMPORT_BY_NAME, IMAGE_IMPORT_DESCRIPTOR,
    IMAGE_NT_SIGNATURE,
};

/// Builds a map of statically imported functions and their IAT addresses
/// by parsing the current module's PE import table.
unsafe fn build_import_map() -> Result<HashMap<String, usize>> {
    let mut imports = HashMap::new();

    // Get base address of current module
    let module = GetModuleHandleA(None)?;
    let module_addr = module.0 as usize;

    // Validate DOS header
    let dos_header = (module_addr as *const IMAGE_DOS_HEADER).read();
    if dos_header.e_magic != IMAGE_DOS_SIGNATURE {
        return Err(Error::from_hresult(ERROR_INVALID_DATA.to_hresult()));
    }

    // Validate NT headers
    let nt_headers_ptr: *const IMAGE_NT_HEADERS32 =
        (module_addr + dos_header.e_lfanew as usize) as _;
    let nt_headers = nt_headers_ptr.read();
    if nt_headers.Signature != IMAGE_NT_SIGNATURE {
        return Err(Error::from_hresult(ERROR_INVALID_DATA.to_hresult()));
    }

    // Get import directory
    let import_directory = nt_headers
        .OptionalHeader
        .DataDirectory
        .get(IMAGE_DIRECTORY_ENTRY_IMPORT.0 as usize)
        .ok_or_else(|| Error::from_hresult(ERROR_INVALID_DATA.to_hresult()))?;

    let import_directory_rva = import_directory.VirtualAddress;
    if import_directory_rva == 0 {
        return Ok(imports); // No imports
    }

    // Iterate through import descriptors
    let mut descriptor_ptr: *const IMAGE_IMPORT_DESCRIPTOR =
        (module_addr as u32 + import_directory_rva) as _;
    let mut descriptor = descriptor_ptr.read();

    while descriptor.Name != 0 {
        // Iterate through thunks for this descriptor
        let mut thunk_ptr: *const u32 = (module_addr as u32 + descriptor.FirstThunk) as _;
        let mut original_thunk_ptr: *const u32 =
            (module_addr as u32 + descriptor.Anonymous.OriginalFirstThunk) as _;
        let mut original_thunk = *original_thunk_ptr;

        while original_thunk != 0 {
            // Get function name from IMAGE_IMPORT_BY_NAME
            let import_by_name_ptr: *const IMAGE_IMPORT_BY_NAME =
                (module_addr as u32 + original_thunk) as _;
            let func_name_ptr = (*import_by_name_ptr).Name.as_ptr();
            let func_name = CStr::from_ptr(func_name_ptr as *const i8)
                .to_string_lossy()
                .to_string();

            // Store IAT address for this function
            imports.insert(func_name, thunk_ptr as usize);

            // Move to next thunk
            thunk_ptr = thunk_ptr.add(1);
            original_thunk_ptr = original_thunk_ptr.add(1);
            original_thunk = *original_thunk_ptr;
        }

        // Move to next import descriptor
        descriptor_ptr = descriptor_ptr.add(1);
        descriptor = descriptor_ptr.read();
    }

    Ok(imports)
}

fn main() -> Result<()> {
    // Build and display the import map
    let import_map = unsafe { build_import_map()? };

    println!("Static imports found: {}", import_map.len());
    for (name, addr) in import_map.iter().take(10) {
        println!("  {}: 0x{:08x}", name, addr);
    }

    if import_map.len() > 10 {
        println!("  ... and {} more", import_map.len() - 10);
    }

    Ok(())
}
