use windows::Win32::System::Diagnostics::Debug::ReadProcessMemory;
use windows::Win32::System::Threading::GetCurrentProcess;

fn call_read_process_memory() -> windows::core::Result<()> {
    // Get current process handle
    let hprocess = unsafe { GetCurrentProcess() };

    // Allocate a buffer for reading
    let mut buffer: [u8; 4] = [0; 4];
    let base_address = 0x0000000000400000 as *const std::ffi::c_void;
    let buffer_ptr = buffer.as_mut_ptr() as *mut std::ffi::c_void;

    // Call ReadProcessMemory with concrete parameters
    // unsafe block is minimal - only the Win32 API call
    unsafe { ReadProcessMemory(hprocess, base_address, buffer_ptr, 4, None) }
}
