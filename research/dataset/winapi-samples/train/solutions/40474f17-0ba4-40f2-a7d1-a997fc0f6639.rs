use std::mem;
use windows::core::{Error, Result, HRESULT};
use windows::Win32::Foundation::ERROR_INVALID_DATA;
use windows::Win32::System::Diagnostics::Debug::{
    IMAGE_DATA_DIRECTORY, IMAGE_DEBUG_DIRECTORY, IMAGE_NT_OPTIONAL_HDR32_MAGIC,
    IMAGE_NT_OPTIONAL_HDR64_MAGIC, IMAGE_OPTIONAL_HEADER32, IMAGE_OPTIONAL_HEADER64,
    IMAGE_SECTION_HEADER,
};
use windows::Win32::System::SystemServices::{
    IMAGE_DOS_HEADER, IMAGE_DOS_SIGNATURE, IMAGE_NT_SIGNATURE,
};

#[derive(Debug, Clone)]
pub struct DebugEntry {
    pub debug_type: u32,
    pub size: u32,
    pub address: u32, // RVA
}

/// Helper to read a structure from a byte slice at a given offset
fn read_struct<T: Copy>(data: &[u8], offset: usize) -> Result<T> {
    let size = mem::size_of::<T>();
    if offset + size > data.len() {
        return Err(Error::from_hresult(HRESULT::from_win32(
            ERROR_INVALID_DATA.0,
        )));
    }
    // SAFETY: We've checked bounds, and T is Copy. The alignment might not be perfect,
    // but for PE parsing we typically read from a byte buffer that may not be aligned.
    // This is acceptable for PE parsing as the structures are designed to be read this way.
    unsafe {
        let ptr = data.as_ptr().add(offset) as *const T;
        Ok(ptr.read_unaligned())
    }
}

/// Convert RVA to file offset using section headers
fn rva_to_offset(sections: &[IMAGE_SECTION_HEADER], rva: u32) -> Option<u32> {
    for section in sections {
        let section_start = section.VirtualAddress;
        // SAFETY: Accessing union field
        let section_end = section_start + unsafe { section.Misc.VirtualSize };
        if rva >= section_start && rva < section_end {
            let offset_in_section = rva - section_start;
            return Some(section.PointerToRawData + offset_in_section);
        }
    }
    None
}

/// Parse debug entries from a PE binary
pub fn parse_debug_entries(data: &[u8]) -> Result<Vec<DebugEntry>> {
    // Check DOS header
    let dos_header: IMAGE_DOS_HEADER = read_struct(data, 0)?;
    if dos_header.e_magic != IMAGE_DOS_SIGNATURE {
        return Err(Error::from_hresult(HRESULT::from_win32(
            ERROR_INVALID_DATA.0,
        )));
    }

    // Get PE header offset
    let pe_offset = dos_header.e_lfanew as usize;

    // Read PE signature
    let pe_signature: u32 = read_struct(data, pe_offset)?;
    if pe_signature != IMAGE_NT_SIGNATURE {
        return Err(Error::from_hresult(HRESULT::from_win32(
            ERROR_INVALID_DATA.0,
        )));
    }

    // Read COFF header (IMAGE_FILE_HEADER) - it's right after the signature
    let coff_offset = pe_offset + 4; // 4 bytes for PE signature

    // Determine if 32-bit or 64-bit by reading the magic from optional header
    let magic_offset = coff_offset + mem::size_of::<u32>() * 3; // Skip Machine, NumberOfSections, TimeDateStamp
    let magic: u16 = read_struct(data, magic_offset)?;

    // Parse based on PE type
    let (
        _optional_header_offset,
        data_directories_offset,
        number_of_rva_and_sizes,
        sections_offset,
    ) = match magic {
        x if x == IMAGE_NT_OPTIONAL_HDR32_MAGIC.0 => {
            let optional_header: IMAGE_OPTIONAL_HEADER32 =
                read_struct(data, coff_offset + mem::size_of::<u32>() * 3)?;
            let data_dir_offset =
                coff_offset + mem::size_of::<u32>() * 3 + mem::size_of::<IMAGE_OPTIONAL_HEADER32>()
                    - mem::size_of::<IMAGE_DATA_DIRECTORY>() * 16;
            (
                coff_offset + mem::size_of::<u32>() * 3,
                data_dir_offset,
                optional_header.NumberOfRvaAndSizes,
                coff_offset + mem::size_of::<u32>() * 3 + mem::size_of::<IMAGE_OPTIONAL_HEADER32>(),
            )
        }
        x if x == IMAGE_NT_OPTIONAL_HDR64_MAGIC.0 => {
            let optional_header: IMAGE_OPTIONAL_HEADER64 =
                read_struct(data, coff_offset + mem::size_of::<u32>() * 3)?;
            let data_dir_offset =
                coff_offset + mem::size_of::<u32>() * 3 + mem::size_of::<IMAGE_OPTIONAL_HEADER64>()
                    - mem::size_of::<IMAGE_DATA_DIRECTORY>() * 16;
            (
                coff_offset + mem::size_of::<u32>() * 3,
                data_dir_offset,
                optional_header.NumberOfRvaAndSizes,
                coff_offset + mem::size_of::<u32>() * 3 + mem::size_of::<IMAGE_OPTIONAL_HEADER64>(),
            )
        }
        _ => {
            return Err(Error::from_hresult(HRESULT::from_win32(
                ERROR_INVALID_DATA.0,
            )));
        }
    };

    // Check if we have enough data directories (need at least 7 for debug directory at index 6)
    if number_of_rva_and_sizes < 7 {
        return Ok(Vec::new());
    }

    // Read debug data directory (index 6)
    let debug_dir_entry_offset =
        data_directories_offset + mem::size_of::<IMAGE_DATA_DIRECTORY>() * 6;
    let debug_data_directory: IMAGE_DATA_DIRECTORY = read_struct(data, debug_dir_entry_offset)?;

    // If debug directory RVA is 0, no debug info
    if debug_data_directory.VirtualAddress == 0 {
        return Ok(Vec::new());
    }

    // Read COFF header to get number of sections
    let number_of_sections: u16 = read_struct(data, coff_offset + 2)?;

    // Read section headers
    let mut sections = Vec::with_capacity(number_of_sections as usize);
    for i in 0..number_of_sections as usize {
        let section_offset = sections_offset + i * mem::size_of::<IMAGE_SECTION_HEADER>();
        let section: IMAGE_SECTION_HEADER = read_struct(data, section_offset)?;
        sections.push(section);
    }

    // Convert debug directory RVA to file offset
    let debug_dir_file_offset = rva_to_offset(&sections, debug_data_directory.VirtualAddress)
        .ok_or_else(|| Error::from_hresult(HRESULT::from_win32(ERROR_INVALID_DATA.0)))?;

    // Calculate number of debug directory entries
    let entry_size = mem::size_of::<IMAGE_DEBUG_DIRECTORY>();
    let num_entries = debug_data_directory.Size as usize / entry_size;

    let mut entries = Vec::with_capacity(num_entries);

    for i in 0..num_entries {
        let entry_offset = debug_dir_file_offset as usize + i * entry_size;
        let debug_dir: IMAGE_DEBUG_DIRECTORY = read_struct(data, entry_offset)?;

        entries.push(DebugEntry {
            debug_type: debug_dir.Type.0,
            size: debug_dir.SizeOfData,
            address: debug_dir.AddressOfRawData,
        });
    }

    Ok(entries)
}
