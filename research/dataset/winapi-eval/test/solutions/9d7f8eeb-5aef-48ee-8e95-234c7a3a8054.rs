use std::iter;
use std::os::windows::ffi::OsStrExt;
use std::path::Path;

use windows::Win32::System::WindowsProgramming::{
    GetPrivateProfileIntW, GetPrivateProfileStringW, WritePrivateProfileStringW,
};
use windows::core::PCWSTR;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AirlockLog {
    pub cycles: u32,
    pub last_badge: String,
}

fn wide(s: &str) -> Vec<u16> {
    s.encode_utf16().chain(iter::once(0)).collect()
}

fn wide_path(path: &Path) -> Vec<u16> {
    path.as_os_str()
        .encode_wide()
        .chain(iter::once(0))
        .collect()
}

fn read_ini_u32(path: &Path, section: &str, key: &str) -> u32 {
    let path_w = wide_path(path);
    let section_w = wide(section);
    let key_w = wide(key);

    unsafe {
        GetPrivateProfileIntW(
            PCWSTR(section_w.as_ptr()),
            PCWSTR(key_w.as_ptr()),
            0,
            PCWSTR(path_w.as_ptr()),
        ) as u32
    }
}

fn read_ini_string(path: &Path, section: &str, key: &str) -> windows::core::Result<String> {
    let path_w = wide_path(path);
    let section_w = wide(section);
    let key_w = wide(key);
    let default_w = wide("");

    let mut capacity = 256usize;

    loop {
        let mut buffer = vec![0u16; capacity];
        let written = unsafe {
            GetPrivateProfileStringW(
                PCWSTR(section_w.as_ptr()),
                PCWSTR(key_w.as_ptr()),
                PCWSTR(default_w.as_ptr()),
                Some(buffer.as_mut_slice()),
                PCWSTR(path_w.as_ptr()),
            ) as usize
        };

        if written < capacity - 1 {
            return Ok(String::from_utf16_lossy(&buffer[..written]));
        }

        capacity *= 2;
    }
}

fn write_ini_value(
    path: &Path,
    section: &str,
    key: &str,
    value: &str,
) -> windows::core::Result<()> {
    let path_w = wide_path(path);
    let section_w = wide(section);
    let key_w = wide(key);
    let value_w = wide(value);

    unsafe {
        WritePrivateProfileStringW(
            PCWSTR(section_w.as_ptr()),
            PCWSTR(key_w.as_ptr()),
            PCWSTR(value_w.as_ptr()),
            PCWSTR(path_w.as_ptr()),
        )
    }
}

fn flush_ini_cache(path: &Path) {
    let path_w = wide_path(path);

    unsafe {
        let _ = WritePrivateProfileStringW(
            PCWSTR(std::ptr::null()),
            PCWSTR(std::ptr::null()),
            PCWSTR(std::ptr::null()),
            PCWSTR(path_w.as_ptr()),
        );
    }
}

pub fn stamp_airlock_log(
    ini_path: &Path,
    airlock: &str,
    badge: &str,
) -> windows::core::Result<AirlockLog> {
    let next_cycles = read_ini_u32(ini_path, airlock, "cycles").saturating_add(1);

    write_ini_value(ini_path, airlock, "last_badge", badge)?;
    write_ini_value(ini_path, airlock, "cycles", &next_cycles.to_string())?;
    flush_ini_cache(ini_path);

    Ok(AirlockLog {
        cycles: read_ini_u32(ini_path, airlock, "cycles"),
        last_badge: read_ini_string(ini_path, airlock, "last_badge")?,
    })
}
