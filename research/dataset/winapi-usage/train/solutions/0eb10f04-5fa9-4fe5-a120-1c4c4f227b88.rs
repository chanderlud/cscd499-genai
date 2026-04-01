use windows::Win32::Foundation::{HANDLE, WIN32_ERROR};
use windows::Win32::System::Diagnostics::Debug::ReadProcessMemory;

unsafe fn call_read_process_memory() -> WIN32_ERROR {
    let hprocess = HANDLE::default();
    let base_address: *mut core::ffi::c_void = std::ptr::null_mut();
    let mut buffer: [u8; 1] = [0; 1];
    let mut size: usize = 1;

    let result = ReadProcessMemory(
        hprocess,
        base_address,
        &mut buffer as *mut _ as *mut _,
        size,
        Some(&mut size),
    );

    match result {
        Ok(_) => WIN32_ERROR(0),
        Err(e) => WIN32_ERROR(e.code().0 as u32),
    }
}
