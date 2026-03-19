use windows::core::{Error, Result, HRESULT};
use windows::Win32::Foundation::{
    CloseHandle, ERROR_FILE_NOT_FOUND, ERROR_PARTIAL_COPY, ERROR_PROC_NOT_FOUND, HANDLE, MAX_PATH,
    UNICODE_STRING,
};
use windows::Win32::System::Diagnostics::Debug::ReadProcessMemory;
use windows::Win32::System::ProcessStatus::{K32EnumProcesses, K32GetModuleFileNameExW};
use windows::Win32::System::Threading::{
    OpenProcess, PROCESS_BASIC_INFORMATION, PROCESS_QUERY_INFORMATION, PROCESS_VM_READ,
};

// Helper function to read a structure from remote process memory
fn read_remote_struct<T: Copy>(process_handle: HANDLE, address: usize) -> Result<T> {
    let mut result = std::mem::MaybeUninit::<T>::uninit();
    let mut bytes_read = 0usize;

    unsafe {
        ReadProcessMemory(
            process_handle,
            address as *const std::ffi::c_void,
            result.as_mut_ptr() as *mut std::ffi::c_void,
            std::mem::size_of::<T>(),
            Some(&mut bytes_read),
        )?;
    }

    if bytes_read != std::mem::size_of::<T>() {
        return Err(Error::from_hresult(ERROR_PARTIAL_COPY.to_hresult()));
    }

    Ok(unsafe { result.assume_init() })
}

// Helper function to read a string from remote process memory
fn read_remote_string(process_handle: HANDLE, address: usize, length_bytes: u16) -> Result<String> {
    let length_chars = (length_bytes as usize) / 2;
    if length_chars == 0 {
        return Ok(String::new());
    }

    let mut buffer = vec![0u16; length_chars];
    let mut bytes_read = 0usize;

    unsafe {
        ReadProcessMemory(
            process_handle,
            address as *const std::ffi::c_void,
            buffer.as_mut_ptr() as *mut std::ffi::c_void,
            length_bytes as usize,
            Some(&mut bytes_read),
        )?;
    }

    if bytes_read != length_bytes as usize {
        return Err(Error::from_hresult(ERROR_PARTIAL_COPY.to_hresult()));
    }

    Ok(String::from_utf16_lossy(&buffer))
}

// Helper function to find process ID by name
fn find_process_id(process_name: &str) -> Result<u32> {
    let mut process_ids = vec![0u32; 1024];
    let mut bytes_returned = 0u32;

    unsafe {
        let success = K32EnumProcesses(
            process_ids.as_mut_ptr(),
            (process_ids.len() * std::mem::size_of::<u32>()) as u32,
            &mut bytes_returned,
        );

        if !success.as_bool() {
            return Err(Error::from_hresult(ERROR_PARTIAL_COPY.to_hresult()));
        }
    }

    let count = bytes_returned as usize / std::mem::size_of::<u32>();
    process_ids.truncate(count);

    let target_name = std::ffi::OsString::from(process_name);

    for pid in process_ids {
        if pid == 0 {
            continue;
        }

        // Try to open the process
        let handle =
            unsafe { OpenProcess(PROCESS_QUERY_INFORMATION | PROCESS_VM_READ, false, pid) };

        if let Ok(handle) = handle {
            // Get the image path for this process
            if let Ok(image_path) = get_process_image_path(handle) {
                let path_name = std::path::Path::new(&image_path);
                if let Some(name) = path_name.file_name() {
                    if name == target_name {
                        unsafe {
                            CloseHandle(handle)?;
                        }
                        return Ok(pid);
                    }
                }
            }

            unsafe {
                CloseHandle(handle)?;
            }
        }
    }

    Err(Error::from_hresult(ERROR_FILE_NOT_FOUND.to_hresult()))
}

// Helper function to get process image path using GetModuleFileNameExW
fn get_process_image_path(process_handle: HANDLE) -> Result<String> {
    let mut buffer = [0u16; MAX_PATH as usize];
    let length = unsafe { K32GetModuleFileNameExW(Some(process_handle), None, &mut buffer) };

    if length == 0 {
        return Err(Error::from_thread());
    }

    Ok(String::from_utf16_lossy(&buffer[..length as usize]))
}

// Main function to get remote process image path from PEB
pub fn get_remote_process_image_path(process_name: &str) -> Result<String> {
    // Step 1: Find the process ID by name
    let pid = find_process_id(process_name)?;

    // Step 2: Open the process with required access rights
    let process_handle =
        unsafe { OpenProcess(PROCESS_QUERY_INFORMATION | PROCESS_VM_READ, false, pid)? };

    // Ensure handle is closed when function exits
    let _handle_guard = scopeguard::guard(process_handle, |h| {
        unsafe {
            let _ = CloseHandle(h);
        };
    });

    // Step 3: Get PEB address using NtQueryInformationProcess
    // We need to dynamically load ntdll.dll and get NtQueryInformationProcess
    let ntdll = unsafe {
        windows::Win32::System::LibraryLoader::GetModuleHandleW(windows::core::w!("ntdll.dll"))?
    };

    let nt_query_information_process = unsafe {
        windows::Win32::System::LibraryLoader::GetProcAddress(
            ntdll,
            windows::core::s!("NtQueryInformationProcess"),
        )
    };

    if nt_query_information_process.is_none() {
        return Err(Error::from_hresult(ERROR_PROC_NOT_FOUND.to_hresult()));
    }

    // Define the function signature for NtQueryInformationProcess
    type NtQueryInformationProcessFn =
        unsafe extern "system" fn(HANDLE, u32, *mut std::ffi::c_void, u32, *mut u32) -> HRESULT;

    let nt_query_information_process: NtQueryInformationProcessFn =
        unsafe { std::mem::transmute(nt_query_information_process.unwrap()) };

    // Call NtQueryInformationProcess to get PROCESS_BASIC_INFORMATION
    let mut process_basic_info = std::mem::MaybeUninit::<PROCESS_BASIC_INFORMATION>::uninit();
    let mut return_length = 0u32;

    let status = unsafe {
        nt_query_information_process(
            process_handle,
            0, // ProcessBasicInformation
            process_basic_info.as_mut_ptr() as *mut std::ffi::c_void,
            std::mem::size_of::<PROCESS_BASIC_INFORMATION>() as u32,
            &mut return_length,
        )
    };

    if status.0 < 0 {
        return Err(Error::from_hresult(status));
    }

    let process_basic_info = unsafe { process_basic_info.assume_init() };

    // Step 4: Read the PEB structure from remote process
    let peb_address = process_basic_info.PebBaseAddress as usize;

    // PEB structure layout (simplified for what we need)
    // We only need to read up to ProcessParameters field
    #[repr(C)]
    #[derive(Copy, Clone)]
    struct PartialPEB {
        inherited_address_space: u8,
        read_image_file_exec_options: u8,
        being_debugged: u8,
        bit_field: u8,
        mutant: usize,
        image_base_address: usize,
        ldr: usize,
        process_parameters: usize,
    }

    let peb: PartialPEB = read_remote_struct(process_handle, peb_address)?;

    // Step 5: Read RTL_USER_PROCESS_PARAMETERS structure
    let process_params_address = peb.process_parameters;

    // RTL_USER_PROCESS_PARAMETERS structure layout (simplified)
    // We need to read up to ImagePathName field
    #[repr(C)]
    #[derive(Copy, Clone)]
    struct PartialProcessParameters {
        maximum_length: u32,
        length: u32,
        flags: u32,
        debug_flags: u32,
        console_handle: usize,
        console_flags: usize,
        standard_input: usize,
        standard_output: usize,
        standard_error: usize,
        current_directory_dos_path: UNICODE_STRING,
        current_directory_handle: usize,
        dll_path: UNICODE_STRING,
        image_path_name: UNICODE_STRING,
    }

    let process_params: PartialProcessParameters =
        read_remote_struct(process_handle, process_params_address)?;

    // Step 6: Read the actual image path string
    let image_path_unicode = process_params.image_path_name;
    let image_path = read_remote_string(
        process_handle,
        image_path_unicode.Buffer.0 as usize,
        image_path_unicode.Length,
    )?;

    Ok(image_path)
}

// Add scopeguard dependency for RAII-style cleanup
// In Cargo.toml: scopeguard = "1.1"
mod scopeguard {
    pub fn guard<T, F: FnOnce(T)>(val: T, f: F) -> ScopeGuard<T, F> {
        ScopeGuard {
            val: Some(val),
            f: Some(f),
        }
    }

    pub struct ScopeGuard<T, F: FnOnce(T)> {
        val: Option<T>,
        f: Option<F>,
    }

    impl<T, F: FnOnce(T)> Drop for ScopeGuard<T, F> {
        fn drop(&mut self) {
            if let (Some(val), Some(f)) = (self.val.take(), self.f.take()) {
                f(val);
            }
        }
    }
}
