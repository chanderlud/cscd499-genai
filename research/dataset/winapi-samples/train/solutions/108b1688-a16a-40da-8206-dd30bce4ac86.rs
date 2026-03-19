use std::ffi::CStr;
use std::mem;
use windows::core::{Error, Result};
use windows::Win32::Foundation::E_FAIL;

#[derive(Debug, Clone, PartialEq)]
pub enum ImportFunction {
    ByName { name: String, hint: u16 },
    ByOrdinal(u16),
}

// PE format constants
const DOS_SIGNATURE: u16 = 0x5A4D; // "MZ"
const NT_SIGNATURE: u32 = 0x00004550; // "PE\0\0"
const IMAGE_FILE_MACHINE_I386: u16 = 0x014c;
const IMAGE_FILE_MACHINE_AMD64: u16 = 0x8664;
const IMAGE_NT_OPTIONAL_HDR32_MAGIC: u16 = 0x10b;
const IMAGE_NT_OPTIONAL_HDR64_MAGIC: u16 = 0x20b;
const IMAGE_NUMBEROF_DIRECTORY_ENTRIES: usize = 16;
const IMAGE_DIRECTORY_ENTRY_IMPORT: usize = 1;

// Helper to read a value from a byte slice at a given offset
fn read_at<T: Copy>(data: &[u8], offset: usize) -> Result<T> {
    let size = mem::size_of::<T>();
    if offset + size > data.len() {
        return Err(Error::from_hresult(E_FAIL));
    }
    // SAFETY: We've checked bounds, and T is Copy. Alignment is not guaranteed,
    // but we're reading from a byte slice which has alignment 1.
    unsafe { Ok(*(data.as_ptr().add(offset) as *const T)) }
}

// Helper to read a null-terminated string from a byte slice
fn read_string_at(data: &[u8], offset: usize) -> Result<String> {
    if offset >= data.len() {
        return Err(Error::from_hresult(E_FAIL));
    }
    let slice = &data[offset..];
    let end = slice.iter().position(|&b| b == 0).unwrap_or(slice.len());
    let cstr = CStr::from_bytes_with_nul(&slice[..=end.min(slice.len() - 1)])
        .map_err(|_| Error::from_hresult(E_FAIL))?;
    cstr.to_str()
        .map(String::from)
        .map_err(|_| Error::from_hresult(E_FAIL))
}

// Convert RVA to file offset using section headers
fn rva_to_offset(sections: &[(u32, u32, u32)], rva: u32) -> Option<u32> {
    for &(virtual_address, virtual_size, raw_offset) in sections {
        if rva >= virtual_address && rva < virtual_address + virtual_size {
            return Some(rva - virtual_address + raw_offset);
        }
    }
    None
}

// Parse import lookup table entry
fn parse_import_entry(
    data: &[u8],
    sections: &[(u32, u32, u32)],
    entry_rva: u32,
    is_64: bool,
) -> Result<Option<ImportFunction>> {
    let entry_offset =
        rva_to_offset(sections, entry_rva).ok_or_else(|| Error::from_hresult(E_FAIL))?;

    if is_64 {
        let value: u64 = read_at(data, entry_offset as usize)?;
        if value == 0 {
            return Ok(None);
        }
        if value & 0x8000000000000000 != 0 {
            // Import by ordinal
            Ok(Some(ImportFunction::ByOrdinal((value & 0xFFFF) as u16)))
        } else {
            // Import by name
            let hint_name_rva = value as u32;
            let hint_name_offset = rva_to_offset(sections, hint_name_rva)
                .ok_or_else(|| Error::from_hresult(E_FAIL))?;
            let hint: u16 = read_at(data, hint_name_offset as usize)?;
            let name = read_string_at(data, (hint_name_offset + 2) as usize)?;
            Ok(Some(ImportFunction::ByName { name, hint }))
        }
    } else {
        let value: u32 = read_at(data, entry_offset as usize)?;
        if value == 0 {
            return Ok(None);
        }
        if value & 0x80000000 != 0 {
            // Import by ordinal
            Ok(Some(ImportFunction::ByOrdinal((value & 0xFFFF) as u16)))
        } else {
            // Import by name
            let hint_name_rva = value;
            let hint_name_offset = rva_to_offset(sections, hint_name_rva)
                .ok_or_else(|| Error::from_hresult(E_FAIL))?;
            let hint: u16 = read_at(data, hint_name_offset as usize)?;
            let name = read_string_at(data, (hint_name_offset + 2) as usize)?;
            Ok(Some(ImportFunction::ByName { name, hint }))
        }
    }
}

pub fn parse_import_table(data: &[u8]) -> Result<Vec<(String, Vec<ImportFunction>)>> {
    // Validate DOS header
    let dos_signature: u16 = read_at(data, 0)?;
    if dos_signature != DOS_SIGNATURE {
        return Err(Error::from_hresult(E_FAIL));
    }

    // Get NT headers offset
    let nt_offset: u32 = read_at(data, 0x3C)?;
    let nt_offset = nt_offset as usize;

    // Validate NT signature
    let nt_signature: u32 = read_at(data, nt_offset)?;
    if nt_signature != NT_SIGNATURE {
        return Err(Error::from_hresult(E_FAIL));
    }

    // Read COFF header
    let machine: u16 = read_at(data, nt_offset + 4)?;
    let number_of_sections: u16 = read_at(data, nt_offset + 6)?;
    let size_of_optional_header: u16 = read_at(data, nt_offset + 20)?;

    // Determine if 32-bit or 64-bit
    let is_64 = match machine {
        IMAGE_FILE_MACHINE_AMD64 => true,
        IMAGE_FILE_MACHINE_I386 => false,
        _ => return Err(Error::from_hresult(E_FAIL)),
    };

    // Read optional header magic
    let optional_header_offset = nt_offset + 24;
    let magic: u16 = read_at(data, optional_header_offset)?;
    let expected_magic = if is_64 {
        IMAGE_NT_OPTIONAL_HDR64_MAGIC
    } else {
        IMAGE_NT_OPTIONAL_HDR32_MAGIC
    };
    if magic != expected_magic {
        return Err(Error::from_hresult(E_FAIL));
    }

    // Get data directory offset
    let data_dir_offset = if is_64 {
        optional_header_offset + 112 // PE32+ data directories start at offset 112
    } else {
        optional_header_offset + 96 // PE32 data directories start at offset 96
    };

    // Check if import directory exists
    if data_dir_offset + (IMAGE_NUMBEROF_DIRECTORY_ENTRIES * 8) > data.len() {
        return Err(Error::from_hresult(E_FAIL));
    }

    // Read import directory entry (RVA and size)
    let import_dir_rva_offset = data_dir_offset + (IMAGE_DIRECTORY_ENTRY_IMPORT * 8);
    let import_rva: u32 = read_at(data, import_dir_rva_offset)?;
    let import_size: u32 = read_at(data, import_dir_rva_offset + 4)?;

    if import_rva == 0 || import_size == 0 {
        return Ok(Vec::new()); // No imports
    }

    // Read section headers
    let section_headers_offset = optional_header_offset + size_of_optional_header as usize;
    let mut sections = Vec::new();
    for i in 0..number_of_sections as usize {
        let offset = section_headers_offset + i * 40; // Each section header is 40 bytes
        let virtual_address: u32 = read_at(data, offset + 12)?;
        let virtual_size: u32 = read_at(data, offset + 8)?;
        let raw_offset: u32 = read_at(data, offset + 20)?;
        sections.push((virtual_address, virtual_size, raw_offset));
    }

    // Convert import directory RVA to file offset
    let import_dir_offset =
        rva_to_offset(&sections, import_rva).ok_or_else(|| Error::from_hresult(E_FAIL))?;

    // Parse import descriptors
    let mut result = Vec::new();
    let mut descriptor_offset = import_dir_offset as usize;

    loop {
        // Read import descriptor fields
        let original_first_thunk: u32 = read_at(data, descriptor_offset)?;
        let name_rva: u32 = read_at(data, descriptor_offset + 12)?;
        let first_thunk: u32 = read_at(data, descriptor_offset + 16)?;

        // Check for null terminator (all fields zero)
        if original_first_thunk == 0 && name_rva == 0 && first_thunk == 0 {
            break;
        }

        // Read DLL name
        let name_offset =
            rva_to_offset(&sections, name_rva).ok_or_else(|| Error::from_hresult(E_FAIL))?;
        let dll_name = read_string_at(data, name_offset as usize)?;

        // Parse import lookup table (use OriginalFirstThunk if present, otherwise FirstThunk)
        let thunk_rva = if original_first_thunk != 0 {
            original_first_thunk
        } else {
            first_thunk
        };
        if thunk_rva == 0 {
            return Err(Error::from_hresult(E_FAIL));
        }

        let thunk_offset =
            rva_to_offset(&sections, thunk_rva).ok_or_else(|| Error::from_hresult(E_FAIL))?;

        let mut functions = Vec::new();
        let mut thunk_index = 0;

        loop {
            let entry_rva_offset = thunk_offset as usize + thunk_index * if is_64 { 8 } else { 4 };
            let entry_rva: u32 = if is_64 {
                let val: u64 = read_at(data, entry_rva_offset)?;
                val as u32
            } else {
                read_at(data, entry_rva_offset)?
            };

            if entry_rva == 0 {
                break;
            }

            if let Some(func) = parse_import_entry(data, &sections, entry_rva, is_64)? {
                functions.push(func);
            }

            thunk_index += 1;
        }

        result.push((dll_name, functions));
        descriptor_offset += 20; // Each import descriptor is 20 bytes
    }

    Ok(result)
}
