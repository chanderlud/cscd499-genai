use std::mem::size_of;
use windows::core::{Error, Result};
use windows::Win32::Foundation::HANDLE;
use windows::Win32::System::Diagnostics::Debug::{
    ReadProcessMemory, IMAGE_DIRECTORY_ENTRY_EXPORT, IMAGE_NT_HEADERS32,
};
use windows::Win32::System::SystemInformation::IMAGE_FILE_MACHINE_I386;
use windows::Win32::System::SystemServices::{IMAGE_DOS_HEADER, IMAGE_EXPORT_DIRECTORY};

fn wide_null(s: &std::ffi::OsStr) -> Vec<u16> {
    use std::{iter::once, os::windows::ffi::OsStrExt};
    s.encode_wide().chain(once(0)).collect()
}

fn read_remote_memory<T: Copy>(handle: HANDLE, address: u32) -> Result<T> {
    let mut buffer = std::mem::MaybeUninit::<T>::uninit();
    let mut bytes_read = 0usize;

    // SAFETY: We're calling ReadProcessMemory with valid parameters.
    // The buffer is valid for writing `size_of::<T>()` bytes.
    unsafe {
        ReadProcessMemory(
            handle,
            address as *const _,
            buffer.as_mut_ptr() as *mut _,
            size_of::<T>(),
            Some(&mut bytes_read),
        )?;
    }

    if bytes_read != size_of::<T>() {
        return Err(Error::from_thread());
    }

    // SAFETY: ReadProcessMemory succeeded and filled the entire buffer.
    Ok(unsafe { buffer.assume_init() })
}

fn read_remote_string(handle: HANDLE, address: u32) -> Result<String> {
    let mut string_bytes = Vec::new();
    let mut current_address = address;

    loop {
        let byte: u8 = read_remote_memory(handle, current_address)?;
        if byte == 0 {
            break;
        }
        string_bytes.push(byte);
        current_address += 1;
    }

    String::from_utf8(string_bytes)
        .map_err(|_| Error::from_hresult(windows::core::HRESULT(0x8007000Du32 as i32)))
    // ERROR_INVALID_DATA
}

pub fn get_remote_module_exports(handle: HANDLE, base_address: u32) -> Result<Vec<(String, u32)>> {
    // Read DOS header
    let dos_header: IMAGE_DOS_HEADER = read_remote_memory(handle, base_address)?;

    // Validate DOS signature
    if dos_header.e_magic != 0x5A4D {
        // "MZ"
        return Err(Error::from_hresult(windows::core::HRESULT(
            0x8007000Bu32 as i32,
        ))); // ERROR_BAD_FORMAT
    }

    // Read NT headers
    let nt_headers_address = base_address + dos_header.e_lfanew as u32;
    let nt_headers: IMAGE_NT_HEADERS32 = read_remote_memory(handle, nt_headers_address)?;

    // Validate NT signature and machine type
    if nt_headers.Signature != 0x00004550 {
        // "PE\0\0"
        return Err(Error::from_hresult(windows::core::HRESULT(
            0x8007000Bu32 as i32,
        ))); // ERROR_BAD_FORMAT
    }

    if nt_headers.FileHeader.Machine != IMAGE_FILE_MACHINE_I386 {
        return Err(Error::from_hresult(windows::core::HRESULT(
            0x8007000Bu32 as i32,
        ))); // ERROR_BAD_FORMAT
    }

    // Get export directory from data directory
    let export_dir_rva = nt_headers.OptionalHeader.DataDirectory
        [IMAGE_DIRECTORY_ENTRY_EXPORT.0 as usize]
        .VirtualAddress;

    // No export directory
    if export_dir_rva == 0 {
        return Ok(Vec::new());
    }

    // Read export directory
    let export_dir: IMAGE_EXPORT_DIRECTORY =
        read_remote_memory(handle, base_address + export_dir_rva)?;

    // Read arrays from export directory
    let names_rva = export_dir.AddressOfNames;
    let functions_rva = export_dir.AddressOfFunctions;
    let ordinals_rva = export_dir.AddressOfNameOrdinals;

    let number_of_names = export_dir.NumberOfNames as usize;

    if number_of_names == 0 {
        return Ok(Vec::new());
    }

    // Read the arrays
    let mut name_rvas = Vec::with_capacity(number_of_names);
    let mut function_rvas = Vec::with_capacity(export_dir.NumberOfFunctions as usize);
    let mut ordinals = Vec::with_capacity(number_of_names);

    for i in 0..number_of_names {
        let name_rva: u32 = read_remote_memory(handle, base_address + names_rva + (i as u32 * 4))?;
        name_rvas.push(name_rva);
    }

    for i in 0..export_dir.NumberOfFunctions as usize {
        let function_rva: u32 =
            read_remote_memory(handle, base_address + functions_rva + (i as u32 * 4))?;
        function_rvas.push(function_rva);
    }

    for i in 0..number_of_names {
        let ordinal: u16 =
            read_remote_memory(handle, base_address + ordinals_rva + (i as u32 * 2))?;
        ordinals.push(ordinal);
    }

    // Build result vector
    let mut exports = Vec::with_capacity(number_of_names);

    for i in 0..number_of_names {
        let name_address = base_address + name_rvas[i];
        let name = read_remote_string(handle, name_address)?;

        let ordinal_index = ordinals[i] as usize;
        if ordinal_index >= function_rvas.len() {
            continue; // Skip invalid ordinal
        }

        let function_rva = function_rvas[ordinal_index];
        exports.push((name, function_rva));
    }

    Ok(exports)
}
