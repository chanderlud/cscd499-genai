use windows::core::Result;
use windows::Win32::Foundation::HWND;
use windows::Win32::System::Diagnostics::Debug::ReadProcessMemory;
use windows::Win32::System::Threading::GetCurrentProcess;

fn get_window_handle() -> Result<HWND> {
    const BASE_ADDRESS: usize = 0xC17054;

    let process_handle = unsafe { GetCurrentProcess() };

    let mut struct_ptr: usize = 0;
    let mut bytes_read: usize = 0;

    unsafe {
        ReadProcessMemory(
            process_handle,
            BASE_ADDRESS as *const std::ffi::c_void,
            &mut struct_ptr as *mut usize as *mut std::ffi::c_void,
            std::mem::size_of::<usize>(),
            Some(&mut bytes_read),
        )?;
    }

    let mut hwnd_value: usize = 0;

    unsafe {
        ReadProcessMemory(
            process_handle,
            struct_ptr as *const std::ffi::c_void,
            &mut hwnd_value as *mut usize as *mut std::ffi::c_void,
            std::mem::size_of::<usize>(),
            Some(&mut bytes_read),
        )?;
    }

    Ok(HWND(hwnd_value as *mut std::ffi::c_void))
}

fn main() {
    if let Ok(hwnd) = get_window_handle() {
        println!("Window handle: {:?}", hwnd);
    } else {
        println!("Failed to get window handle");
    }
}
