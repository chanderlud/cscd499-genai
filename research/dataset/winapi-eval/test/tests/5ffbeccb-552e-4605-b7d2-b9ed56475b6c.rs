#[cfg(test)]
mod tests {
    use super::*;
    use windows::Win32::Foundation::CloseHandle;
    use windows::Win32::System::Memory::{
        FILE_MAP_READ, MapViewOfFile, OpenFileMappingW, UnmapViewOfFile,
    };
    use windows::core::{Error, PCWSTR, Result};

    fn read_beacon_bytes(name: &str) -> Result<[u8; MAPPING_SIZE]> {
        let mapping_name = format!("Local\\Beacon::{name}");
        let wide_name: Vec<u16> = mapping_name
            .encode_utf16()
            .chain(std::iter::once(0))
            .collect();

        let handle =
            unsafe { OpenFileMappingW(FILE_MAP_READ.0, false, PCWSTR(wide_name.as_ptr()))? };
        let view = unsafe { MapViewOfFile(handle, FILE_MAP_READ, 0, 0, MAPPING_SIZE) };
        if view.Value.is_null() {
            unsafe {
                let _ = CloseHandle(handle);
            }
            return Err(Error::from_thread());
        }

        let mut out = [0u8; MAPPING_SIZE];
        unsafe {
            let src = std::slice::from_raw_parts(view.Value as *const u8, MAPPING_SIZE);
            out.copy_from_slice(src);
            let _ = UnmapViewOfFile(view);
            let _ = CloseHandle(handle);
        }
        Ok(out)
    }

    #[test]
    fn creates_mapping_and_writes_expected_layout() -> Result<()> {
        let info = publish_beacon("unit_basic", "ready")?;
        assert_eq!(info.mapping_name, "Local\\Beacon::unit_basic");
        assert_eq!(info.payload_len, 5);
        assert_eq!(
            info.checksum,
            b"ready".iter().fold(0u32, |a, b| a.wrapping_add(*b as u32))
        );

        let bytes = read_beacon_bytes("unit_basic")?;
        assert_eq!(&bytes[0..4], b"BCN1");
        assert_eq!(u32::from_le_bytes(bytes[4..8].try_into().unwrap()), 5);
        assert_eq!(
            u32::from_le_bytes(bytes[8..12].try_into().unwrap()),
            b"ready".iter().fold(0u32, |a, b| a.wrapping_add(*b as u32))
        );
        assert_eq!(&bytes[12..17], b"ready");
        Ok(())
    }

    #[test]
    fn second_write_overwrites_and_zero_fills_tail() -> Result<()> {
        publish_beacon("unit_overwrite", "abcdefghij")?;
        let info = publish_beacon("unit_overwrite", "xy")?;
        assert!(!info.newly_created);

        let bytes = read_beacon_bytes("unit_overwrite")?;
        assert_eq!(&bytes[0..4], b"BCN1");
        assert_eq!(u32::from_le_bytes(bytes[4..8].try_into().unwrap()), 2);
        assert_eq!(&bytes[12..14], b"xy");

        for b in &bytes[14..22] {
            assert_eq!(*b, 0);
        }
        Ok(())
    }

    #[test]
    fn rejects_too_large_payload() {
        let oversized = "x".repeat(MAX_PAYLOAD + 1);
        let result = publish_beacon("unit_too_large", &oversized);
        assert!(result.is_err());
    }
}