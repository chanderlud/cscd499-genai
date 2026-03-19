use windows::core::{Error, Result, HRESULT};

#[derive(Debug, Clone)]
pub struct RelocationEntry {
    pub relocation_type: u8,
    pub offset: u16,
}

#[derive(Debug, Clone)]
pub struct RelocationBlock {
    pub page_rva: u32,
    pub entries: Vec<RelocationEntry>,
}

const DOS_SIGNATURE: u16 = 0x5A4D; // "MZ"
const NT_SIGNATURE: u32 = 0x00004550; // "PE\0\0"
const IMAGE_DIRECTORY_ENTRY_BASERELOC: usize = 5;

fn read_u16(data: &[u8], offset: usize) -> Result<u16> {
    data.get(offset..offset + 2)
        .ok_or_else(|| Error::new(HRESULT::from_win32(0x80070057), "Invalid offset for u16"))
        .map(|bytes| u16::from_le_bytes(bytes.try_into().unwrap()))
}

fn read_u32(data: &[u8], offset: usize) -> Result<u32> {
    data.get(offset..offset + 4)
        .ok_or_else(|| Error::new(HRESULT::from_win32(0x80070057), "Invalid offset for u32"))
        .map(|bytes| u32::from_le_bytes(bytes.try_into().unwrap()))
}

fn rva_to_file_offset(rva: u32, sections: &[(u32, u32, u32)]) -> Result<usize> {
    for &(section_rva, section_size, section_offset) in sections {
        if rva >= section_rva && rva < section_rva + section_size {
            return Ok((rva - section_rva + section_offset) as usize);
        }
    }
    Err(Error::new(
        HRESULT::from_win32(0x8007000B),
        "RVA not found in any section",
    ))
}

pub fn parse_base_relocations(data: &[u8]) -> Result<Vec<RelocationBlock>> {
    let dos_signature = read_u16(data, 0)?;
    if dos_signature != DOS_SIGNATURE {
        return Err(Error::new(
            HRESULT::from_win32(0x8007000B),
            "Invalid DOS signature",
        ));
    }

    let nt_offset = read_u32(data, 0x3C)? as usize;

    let nt_signature = read_u32(data, nt_offset)?;
    if nt_signature != NT_SIGNATURE {
        return Err(Error::new(
            HRESULT::from_win32(0x8007000B),
            "Invalid NT signature",
        ));
    }

    let magic_offset = nt_offset + 24;
    let magic = read_u16(data, magic_offset)?;

    let data_dir_offset = match magic {
        0x10B => magic_offset + 96,  // PE32
        0x20B => magic_offset + 112, // PE32+
        _ => {
            return Err(Error::new(
                HRESULT::from_win32(0x8007000B),
                "Invalid PE magic",
            ))
        }
    };

    let number_of_sections = read_u16(data, nt_offset + 6)?;
    let size_of_optional_header = read_u16(data, nt_offset + 20)?;

    let section_headers_offset = nt_offset + 24 + size_of_optional_header as usize;
    let mut sections = Vec::new();

    for i in 0..number_of_sections as usize {
        let section_offset = section_headers_offset + i * 40;
        let virtual_size = read_u32(data, section_offset + 8)?;
        let virtual_address = read_u32(data, section_offset + 12)?;
        let raw_data_offset = read_u32(data, section_offset + 20)?;
        sections.push((virtual_address, virtual_size, raw_data_offset));
    }

    let reloc_dir_offset = data_dir_offset + IMAGE_DIRECTORY_ENTRY_BASERELOC * 8;
    let reloc_rva = read_u32(data, reloc_dir_offset)?;
    let reloc_size = read_u32(data, reloc_dir_offset + 4)?;

    if reloc_rva == 0 || reloc_size == 0 {
        return Ok(Vec::new());
    }

    let reloc_file_offset = rva_to_file_offset(reloc_rva, &sections)?;

    let mut blocks = Vec::new();
    let mut current_offset = reloc_file_offset;
    let end_offset = reloc_file_offset + reloc_size as usize;

    while current_offset < end_offset {
        let page_rva = read_u32(data, current_offset)?;
        let block_size = read_u32(data, current_offset + 4)?;

        if block_size < 8 {
            return Err(Error::new(
                HRESULT::from_win32(0x8007000B),
                "Invalid relocation block size",
            ));
        }

        let entry_count = (block_size as usize - 8) / 2;
        let mut entries = Vec::with_capacity(entry_count);

        for i in 0..entry_count {
            let entry_offset = current_offset + 8 + i * 2;
            let entry_data = read_u16(data, entry_offset)?;

            let relocation_type = (entry_data >> 12) as u8;
            let offset = entry_data & 0x0FFF;

            entries.push(RelocationEntry {
                relocation_type,
                offset,
            });
        }

        blocks.push(RelocationBlock { page_rva, entries });

        current_offset += block_size as usize;
    }

    Ok(blocks)
}
