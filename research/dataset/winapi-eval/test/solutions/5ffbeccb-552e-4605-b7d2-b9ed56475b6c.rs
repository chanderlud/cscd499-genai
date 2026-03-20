use windows::Win32::Foundation::{
    ERROR_ALREADY_EXISTS, GetLastError, HANDLE, INVALID_HANDLE_VALUE,
};
use windows::Win32::System::Memory::{
    CreateFileMappingW, FILE_MAP_READ, FILE_MAP_WRITE, MEMORY_MAPPED_VIEW_ADDRESS, MapViewOfFile,
    PAGE_READWRITE, UnmapViewOfFile,
};
use windows::core::{Error, PCWSTR, Result};

const MAPPING_SIZE: usize = 4096;
const HEADER_SIZE: usize = 12;
const MAX_PAYLOAD: usize = MAPPING_SIZE - HEADER_SIZE;
const MAGIC: &[u8; 4] = b"BCN1";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BeaconInfo {
    pub mapping_name: String,
    pub payload_len: u32,
    pub checksum: u32,
    pub newly_created: bool,
    pub handle: HANDLE,
}

fn checksum(bytes: &[u8]) -> u32 {
    bytes
        .iter()
        .fold(0u32, |acc, &b| acc.wrapping_add(b as u32))
}

struct ViewPtr(MEMORY_MAPPED_VIEW_ADDRESS);

impl Drop for ViewPtr {
    fn drop(&mut self) {
        unsafe {
            let _ = UnmapViewOfFile(self.0);
        }
    }
}

pub fn publish_beacon(name: &str, payload: &str) -> Result<BeaconInfo> {
    if name.is_empty() {
        return Err(Error::new(
            windows::core::HRESULT(0x80070057u32 as i32),
            "name must not be empty",
        ));
    }

    let payload_bytes = payload.as_bytes();
    if payload_bytes.len() > MAX_PAYLOAD {
        return Err(Error::new(
            windows::core::HRESULT(0x80070057u32 as i32),
            "payload exceeds 4084 bytes",
        ));
    }

    let mapping_name = format!("Local\\Beacon::{name}");
    let wide_name: Vec<u16> = mapping_name
        .encode_utf16()
        .chain(std::iter::once(0))
        .collect();

    let handle = unsafe {
        CreateFileMappingW(
            INVALID_HANDLE_VALUE,
            None,
            PAGE_READWRITE,
            0,
            MAPPING_SIZE as u32,
            PCWSTR(wide_name.as_ptr()),
        )?
    };

    let newly_created = unsafe { GetLastError() } != ERROR_ALREADY_EXISTS;

    let view = unsafe { MapViewOfFile(handle, FILE_MAP_WRITE | FILE_MAP_READ, 0, 0, MAPPING_SIZE) };
    if view.Value.is_null() {
        return Err(Error::from_thread());
    }
    let view = ViewPtr(view);

    let checksum = checksum(payload_bytes);

    unsafe {
        let buf = std::slice::from_raw_parts_mut(view.0.Value as *mut u8, MAPPING_SIZE);
        buf.fill(0);

        buf[0..4].copy_from_slice(MAGIC);
        buf[4..8].copy_from_slice(&(payload_bytes.len() as u32).to_le_bytes());
        buf[8..12].copy_from_slice(&checksum.to_le_bytes());
        buf[12..12 + payload_bytes.len()].copy_from_slice(payload_bytes);
    }

    Ok(BeaconInfo {
        mapping_name,
        payload_len: payload_bytes.len() as u32,
        checksum,
        newly_created,
        handle,
    })
}
