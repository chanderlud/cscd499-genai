use std::ffi::OsStr;
use std::iter::once;
use std::os::windows::ffi::OsStrExt;
use windows::core::{Error, Result, PCSTR, PCWSTR};
use windows::Win32::Foundation::{CloseHandle, HANDLE, INVALID_HANDLE_VALUE};
use windows::Win32::System::Diagnostics::Debug::WriteProcessMemory;
use windows::Win32::System::LibraryLoader::{GetModuleHandleW, GetProcAddress};
use windows::Win32::System::Memory::{VirtualAllocEx, MEM_COMMIT, MEM_RESERVE, PAGE_READWRITE};
use windows::Win32::System::Threading::{
    CreateRemoteThreadEx, OpenProcess, WaitForSingleObject, INFINITE, PROCESS_ALL_ACCESS,
};

fn wide_null(s: &str) -> Vec<u16> {
    OsStr::new(s).encode_wide().chain(once(0)).collect()
}

fn inject_dll(process_id: u32, dll_path: &str) -> Result<()> {
    // Convert DLL path to wide string with null terminator
    let dll_path_wide = wide_null(dll_path);

    // Open target process with full access
    let process_handle = unsafe { OpenProcess(PROCESS_ALL_ACCESS, false, process_id)? };

    // Ensure we close the process handle on any error path
    let process_guard = ProcessHandleGuard(process_handle);

    // Allocate memory in remote process for DLL path
    let remote_memory = unsafe {
        VirtualAllocEx(
            process_handle,
            None,
            dll_path_wide.len() * 2, // Size in bytes (wide chars are 2 bytes)
            MEM_COMMIT | MEM_RESERVE,
            PAGE_READWRITE,
        )
    };

    if remote_memory.is_null() {
        return Err(Error::from_thread());
    }

    // Write DLL path to remote process memory
    let mut bytes_written = 0usize;
    let success = unsafe {
        WriteProcessMemory(
            process_handle,
            remote_memory,
            dll_path_wide.as_ptr() as *const _,
            dll_path_wide.len() * 2,
            Some(&mut bytes_written),
        )
    };

    if success.is_err() {
        return Err(Error::from_thread());
    }

    // Get address of LoadLibraryW from kernel32.dll
    let kernel32_module = unsafe { GetModuleHandleW(PCWSTR(wide_null("kernel32.dll").as_ptr()))? };
    let load_library_addr = unsafe {
        GetProcAddress(kernel32_module, PCSTR(b"LoadLibraryW\0".as_ptr()))
            .ok_or_else(Error::from_thread)?
    };

    // Create remote thread to execute LoadLibraryW with our DLL path
    let thread_handle = unsafe {
        CreateRemoteThreadEx(
            process_handle,
            None,
            0,
            Some(std::mem::transmute(load_library_addr)),
            Some(remote_memory),
            0,
            None,
            None,
        )?
    };

    // Ensure we close the thread handle
    let thread_guard = ThreadHandleGuard(thread_handle);

    // Wait for remote thread to complete
    unsafe {
        WaitForSingleObject(thread_handle, INFINITE);
    }

    // Guards will automatically clean up handles when they go out of scope
    Ok(())
}

// RAII guard for process handle
struct ProcessHandleGuard(HANDLE);

impl Drop for ProcessHandleGuard {
    fn drop(&mut self) {
        if self.0 != INVALID_HANDLE_VALUE {
            unsafe {
                let _ = CloseHandle(self.0);
            }
        }
    }
}

// RAII guard for thread handle
struct ThreadHandleGuard(HANDLE);

impl Drop for ThreadHandleGuard {
    fn drop(&mut self) {
        if self.0 != INVALID_HANDLE_VALUE {
            unsafe {
                let _ = CloseHandle(self.0);
            }
        }
    }
}
