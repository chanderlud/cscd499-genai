use std::ffi::OsStr;
use std::iter::once;
use std::os::windows::ffi::OsStrExt;
use std::path::Path;
use windows::core::{Error, Result, HRESULT, PCWSTR};
use windows::Win32::Foundation::{CloseHandle, ERROR_MOD_NOT_FOUND, HANDLE};
use windows::Win32::Storage::FileSystem::{
    CreateFileW, WriteFile, FILE_ATTRIBUTE_NORMAL, FILE_GENERIC_WRITE, FILE_SHARE_MODE, OPEN_ALWAYS,
};
use windows::Win32::System::Diagnostics::Debug::ReadProcessMemory;
use windows::Win32::System::Diagnostics::ToolHelp::{
    CreateToolhelp32Snapshot, Module32FirstW, Module32NextW, MODULEENTRY32W, TH32CS_SNAPMODULE,
    TH32CS_SNAPMODULE32,
};
use windows::Win32::System::Threading::{OpenProcess, PROCESS_QUERY_INFORMATION, PROCESS_VM_READ};

fn wide_null(s: &std::ffi::OsStr) -> Vec<u16> {
    s.encode_wide().chain(once(0)).collect()
}

// Simple RAII guard for HANDLE cleanup
struct HandleGuard(HANDLE);
impl Drop for HandleGuard {
    fn drop(&mut self) {
        unsafe {
            let _ = CloseHandle(self.0);
        };
    }
}

fn dump_remote_module(pid: u32, module_name: &str) -> Result<()> {
    // Open the target process with required access rights
    let process_handle =
        unsafe { OpenProcess(PROCESS_QUERY_INFORMATION | PROCESS_VM_READ, false, pid) }?;
    let _process_guard = HandleGuard(process_handle);

    // Create a snapshot of the process's modules
    let snapshot =
        unsafe { CreateToolhelp32Snapshot(TH32CS_SNAPMODULE | TH32CS_SNAPMODULE32, pid) }?;
    let _snapshot_guard = HandleGuard(snapshot);

    // Initialize module entry structure
    let mut module_entry = MODULEENTRY32W::default();
    module_entry.dwSize = std::mem::size_of::<MODULEENTRY32W>() as u32;

    // Get the first module in the snapshot
    let found = unsafe { Module32FirstW(snapshot, &mut module_entry) }.is_ok();
    if !found {
        return Err(Error::from_hresult(HRESULT::from_win32(
            ERROR_MOD_NOT_FOUND.0,
        )));
    }

    // Iterate through modules to find the target
    loop {
        // Compare module names (case-insensitive)
        let current_name = unsafe {
            let len = module_entry
                .szModule
                .iter()
                .position(|&c| c == 0)
                .unwrap_or(module_entry.szModule.len());
            String::from_utf16_lossy(&module_entry.szModule[..len])
        };

        if current_name.eq_ignore_ascii_case(module_name) {
            // Found the target module
            let base_address = module_entry.modBaseAddr;
            let module_size = module_entry.modBaseSize as usize;

            // Allocate buffer for module memory
            let mut buffer = vec![0u8; module_size];
            let mut bytes_read = 0usize;

            // Read the module's memory
            let success = unsafe {
                ReadProcessMemory(
                    process_handle,
                    base_address as *const _,
                    buffer.as_mut_ptr() as *mut _,
                    module_size,
                    Some(&mut bytes_read),
                )
            }
            .is_ok();

            if !success {
                return Err(Error::from_thread());
            }

            // Create output file
            let output_filename = format!("{}.dump", module_name);
            let output_path = Path::new(&output_filename);
            let file_handle = unsafe {
                CreateFileW(
                    PCWSTR(wide_null(output_path.as_os_str()).as_ptr()),
                    FILE_GENERIC_WRITE.0,
                    FILE_SHARE_MODE(0),
                    None,
                    OPEN_ALWAYS,
                    FILE_ATTRIBUTE_NORMAL,
                    None,
                )
            }?;
            let _file_guard = HandleGuard(file_handle);

            // Write the buffer to file
            let mut bytes_written = 0u32;
            unsafe {
                WriteFile(file_handle, Some(&buffer), Some(&mut bytes_written), None)?;
            }

            return Ok(());
        }

        // Move to next module
        let next_found = unsafe { Module32NextW(snapshot, &mut module_entry) }.is_ok();
        if !next_found {
            break;
        }
    }

    // Module not found
    Err(Error::from_hresult(HRESULT::from_win32(
        ERROR_MOD_NOT_FOUND.0,
    )))
}
