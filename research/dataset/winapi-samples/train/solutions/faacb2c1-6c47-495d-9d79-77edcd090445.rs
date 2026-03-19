use std::ffi::OsStr;
use std::fs::File;
use std::io::Write;
use std::iter::once;
use std::os::windows::ffi::OsStrExt;
use windows::core::{Error, Result, HRESULT, PCWSTR};
use windows::Win32::Foundation::{CloseHandle, ERROR_NO_MORE_FILES, HANDLE, INVALID_HANDLE_VALUE};
use windows::Win32::System::Diagnostics::ToolHelp::{
    CreateToolhelp32Snapshot, Process32FirstW, Process32NextW, PROCESSENTRY32W, TH32CS_SNAPPROCESS,
};
use windows::Win32::System::Memory::{VirtualQueryEx, MEMORY_BASIC_INFORMATION, MEM_COMMIT};
use windows::Win32::System::Threading::{OpenProcess, PROCESS_QUERY_INFORMATION, PROCESS_VM_READ};

fn wide_null(s: &OsStr) -> Vec<u16> {
    s.encode_wide().chain(once(0)).collect()
}

fn get_process_id_by_name(process_name: &str) -> Result<u32> {
    let snapshot = unsafe { CreateToolhelp32Snapshot(TH32CS_SNAPPROCESS, 0) }?;
    let mut process_entry = PROCESSENTRY32W {
        dwSize: std::mem::size_of::<PROCESSENTRY32W>() as u32,
        ..Default::default()
    };

    let mut found = false;
    // SAFETY: The snapshot handle is valid, and process_entry is properly initialized.
    unsafe { Process32FirstW(snapshot, &mut process_entry) }?;

    loop {
        let current_name = String::from_utf16_lossy(
            &process_entry.szExeFile[..process_entry
                .szExeFile
                .iter()
                .position(|&c| c == 0)
                .unwrap_or(0)],
        );
        if current_name.eq_ignore_ascii_case(process_name) {
            found = true;
            break;
        }

        // SAFETY: The snapshot handle is still valid.
        match unsafe { Process32NextW(snapshot, &mut process_entry) } {
            Ok(_) => {}
            Err(e) if e.code() == HRESULT::from_win32(ERROR_NO_MORE_FILES.0) => break,
            Err(e) => return Err(e),
        }
    }

    // SAFETY: The snapshot handle was created successfully.
    unsafe { CloseHandle(snapshot) }?;

    if found {
        Ok(process_entry.th32ProcessID)
    } else {
        Err(Error::from_hresult(HRESULT::from_win32(
            windows::Win32::Foundation::ERROR_NOT_FOUND.0,
        )))
    }
}

pub fn log_process_memory_regions(process_name: &str, output_path: &str) -> Result<()> {
    let process_id = get_process_id_by_name(process_name)?;

    // SAFETY: We have a valid process ID and request appropriate access rights.
    let process_handle = unsafe {
        OpenProcess(
            PROCESS_QUERY_INFORMATION | PROCESS_VM_READ,
            false,
            process_id,
        )
    }?;

    let mut file = File::create(output_path).map_err(|e| {
        Error::from_hresult(HRESULT::from_win32(
            windows::Win32::Foundation::ERROR_CANNOT_MAKE.0,
        ))
    })?;

    let mut address = 0usize;
    let mut memory_info = MEMORY_BASIC_INFORMATION::default();

    loop {
        // SAFETY: process_handle is valid, memory_info is properly aligned and sized.
        let result = unsafe {
            VirtualQueryEx(
                process_handle,
                Some(address as *const std::ffi::c_void),
                &mut memory_info,
                std::mem::size_of::<MEMORY_BASIC_INFORMATION>(),
            )
        };

        if result == 0 {
            break;
        }

        if memory_info.State == MEM_COMMIT {
            let base_address = memory_info.BaseAddress as usize;
            let region_size = memory_info.RegionSize;
            let end_address = base_address + region_size;
            let protection = memory_info.Protect.0;

            writeln!(
                file,
                "0x{:X} - 0x{:X} ({} bytes) Protection: {:X}",
                base_address, end_address, region_size, protection
            )
            .map_err(|e| {
                Error::from_hresult(HRESULT::from_win32(
                    windows::Win32::Foundation::ERROR_WRITE_FAULT.0,
                ))
            })?;
        }

        address = memory_info.BaseAddress as usize + memory_info.RegionSize;
    }

    // SAFETY: process_handle was successfully opened.
    unsafe { CloseHandle(process_handle) }?;

    Ok(())
}
