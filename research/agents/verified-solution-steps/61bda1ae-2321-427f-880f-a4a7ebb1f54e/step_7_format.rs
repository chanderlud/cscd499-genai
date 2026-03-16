use std::ffi::OsStr;
use std::iter::once;
use std::os::windows::ffi::OsStrExt;
use std::path::Path;
use windows::core::{Error, Result, PCWSTR};
use windows::Win32::Foundation::FreeLibrary;
use windows::Win32::Foundation::ERROR_BAD_FORMAT;
use windows::Win32::System::Diagnostics::Debug::{
    IMAGE_NT_HEADERS32, IMAGE_NT_HEADERS64, IMAGE_NT_OPTIONAL_HDR32_MAGIC,
    IMAGE_NT_OPTIONAL_HDR64_MAGIC,
};
use windows::Win32::System::LibraryLoader::{LoadLibraryExW, DONT_RESOLVE_DLL_REFERENCES};
use windows::Win32::System::SystemServices::{IMAGE_DOS_HEADER, IMAGE_EXPORT_DIRECTORY};

fn wide_null(s: &OsStr) -> Vec<u16> {
    s.encode_wide().chain(once(0)).collect()
}

fn rva_to_va(base: *const u8, rva: u32) -> *const u8 {
    unsafe { base.add(rva as usize) }
}

fn read_string_from_rva(base: *const u8, rva: u32) -> String {
    let ptr = rva_to_va(base, rva);
    let mut len = 0;
    unsafe {
        while *ptr.add(len) != 0 {
            len += 1;
        }
        String::from_utf8_lossy(std::slice::from_raw_parts(ptr, len)).to_string()
    }
}

fn get_export_directory(base: *const u8) -> Result<(*const IMAGE_EXPORT_DIRECTORY, u32)> {
    unsafe {
        let dos_header = base as *const IMAGE_DOS_HEADER;
        if (*dos_header).e_magic != 0x5A4D {
            // "MZ"
            return Err(Error::from_hresult(windows::core::HRESULT::from_win32(
                ERROR_BAD_FORMAT.0,
            )));
        }

        let e_lfanew = (*dos_header).e_lfanew as usize;
        let nt_headers_ptr = base.add(e_lfanew);

        // Read signature
        let signature = *(nt_headers_ptr as *const u32);
        if signature != 0x00004550 {
            // "PE\0\0"
            return Err(Error::from_hresult(windows::core::HRESULT::from_win32(
                ERROR_BAD_FORMAT.0,
            )));
        }

        // Read optional header magic (first 2 bytes of optional header)
        // Optional header starts after signature (4 bytes) and file header (20 bytes)
        let magic_ptr = nt_headers_ptr.add(24) as *const u16;
        let magic = *magic_ptr;

        let data_directory = if magic == IMAGE_NT_OPTIONAL_HDR32_MAGIC.0 {
            let nt_headers = &*(nt_headers_ptr as *const IMAGE_NT_HEADERS32);
            &nt_headers.OptionalHeader.DataDirectory
        } else if magic == IMAGE_NT_OPTIONAL_HDR64_MAGIC.0 {
            let nt_headers = &*(nt_headers_ptr as *const IMAGE_NT_HEADERS64);
            &nt_headers.OptionalHeader.DataDirectory
        } else {
            return Err(Error::from_hresult(windows::core::HRESULT::from_win32(
                ERROR_BAD_FORMAT.0,
            )));
        };

        let export_entry = &data_directory[0]; // IMAGE_DIRECTORY_ENTRY_EXPORT = 0
        if export_entry.VirtualAddress == 0 || export_entry.Size == 0 {
            return Err(Error::from_hresult(windows::core::HRESULT::from_win32(
                ERROR_BAD_FORMAT.0,
            )));
        }

        let export_dir =
            rva_to_va(base, export_entry.VirtualAddress) as *const IMAGE_EXPORT_DIRECTORY;
        Ok((export_dir, export_entry.Size))
    }
}

fn resolve_forwarder(base: *const u8, forwarder_rva: u32) -> Result<usize> {
    let forwarder_str = read_string_from_rva(base, forwarder_rva);
    let parts: Vec<&str> = forwarder_str.splitn(2, '.').collect();
    if parts.len() != 2 {
        return Err(Error::from_hresult(windows::core::HRESULT::from_win32(
            ERROR_BAD_FORMAT.0,
        )));
    }

    let dll_name = parts[0];
    let export_name = parts[1];

    let dll_path = Path::new(dll_name);
    resolve_export_manual(dll_path, export_name)
}

fn find_symbol_in_export(
    export_dir: *const IMAGE_EXPORT_DIRECTORY,
    base: *const u8,
    symbol: &str,
) -> Result<usize> {
    unsafe {
        let number_of_names = (*export_dir).NumberOfNames;
        let address_of_names = (*export_dir).AddressOfNames;
        let address_of_name_ordinals = (*export_dir).AddressOfNameOrdinals;
        let address_of_functions = (*export_dir).AddressOfFunctions;

        let names_ptr = rva_to_va(base, address_of_names) as *const u32;
        let ordinals_ptr = rva_to_va(base, address_of_name_ordinals) as *const u16;
        let functions_ptr = rva_to_va(base, address_of_functions) as *const u32;

        for i in 0..number_of_names as usize {
            let name_rva = *names_ptr.add(i);
            let name = read_string_from_rva(base, name_rva);

            if name == symbol {
                let ordinal = *ordinals_ptr.add(i) as usize;
                let function_rva = *functions_ptr.add(ordinal);

                let export_dir_rva = (export_dir as usize - base as usize) as u32;
                let export_dir_size = (*export_dir).NumberOfFunctions * 4; // Approximate size

                if function_rva >= export_dir_rva && function_rva < export_dir_rva + export_dir_size
                {
                    // This is a forwarder
                    return resolve_forwarder(base, function_rva);
                } else {
                    // Regular export
                    return Ok(base.add(function_rva as usize) as usize);
                }
            }
        }

        Err(Error::from_hresult(windows::core::HRESULT::from_win32(
            ERROR_BAD_FORMAT.0,
        )))
    }
}

pub fn resolve_export_manual(dll: &Path, symbol: &str) -> Result<usize> {
    let dll_path_wide = wide_null(dll.as_os_str());

    let hmodule = unsafe {
        LoadLibraryExW(
            PCWSTR(dll_path_wide.as_ptr()),
            None,
            DONT_RESOLVE_DLL_REFERENCES,
        )
    }?;

    let base = hmodule.0 as *const u8;

    let result = (|| -> Result<usize> {
        let (export_dir, _export_size) = get_export_directory(base)?;
        find_symbol_in_export(export_dir, base, symbol)
    })();

    unsafe { FreeLibrary(hmodule) };

    result
}