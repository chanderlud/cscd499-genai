use std::ffi::{c_void, CString};
use std::ptr;
use windows::core::{Error, Result, PCSTR};
use windows::Win32::Foundation::{CloseHandle, INVALID_HANDLE_VALUE, LPARAM, WPARAM};
use windows::Win32::System::DataExchange::COPYDATASTRUCT;
use windows::Win32::System::Memory::{
    CreateFileMappingA, MapViewOfFile, UnmapViewOfFile, FILE_MAP_ALL_ACCESS, PAGE_READWRITE,
};
use windows::Win32::UI::WindowsAndMessaging::{FindWindowA, SendMessageA, WM_COPYDATA};

const PUTTY_IPC_MAGIC: usize = 0x804e50ba;
const PUTTY_IPC_MAXLEN: usize = 16384;

fn main() -> Result<()> {
    // Create a named shared memory region
    let map_name = CString::new("gpg_bridge_test").unwrap();
    let handle = unsafe {
        CreateFileMappingA(
            INVALID_HANDLE_VALUE,
            None,
            PAGE_READWRITE,
            0,
            PUTTY_IPC_MAXLEN as u32,
            PCSTR::from_raw(map_name.as_ptr() as *const u8),
        )
    }?;

    // Map the shared memory into our address space
    let view = unsafe { MapViewOfFile(handle, FILE_MAP_ALL_ACCESS, 0, 0, PUTTY_IPC_MAXLEN) };
    if view.Value.is_null() {
        unsafe {
            let _ = CloseHandle(handle);
        }
        return Err(Error::from_thread());
    }

    // Write a test message to shared memory (length-prefixed)
    let message = b"Hello Pageant!";
    let len = message.len() as u32;
    unsafe {
        // Write length in big-endian format
        (view.Value as *mut u32).write_unaligned(len.to_be());
        // Write message data
        ptr::copy_nonoverlapping(
            message.as_ptr(),
            (view.Value as *mut u8).add(4),
            message.len(),
        );
    }

    // Find the Pageant window
    let window_name = CString::new("Pageant").unwrap();
    let window_class = CString::new("Pageant").unwrap();
    let hwnd = unsafe {
        FindWindowA(
            PCSTR::from_raw(window_class.as_ptr() as *const u8),
            PCSTR::from_raw(window_name.as_ptr() as *const u8),
        )
    }?;

    // Prepare COPYDATASTRUCT with shared memory name
    let mut copy_data = COPYDATASTRUCT {
        dwData: PUTTY_IPC_MAGIC,
        cbData: map_name.as_bytes_with_nul().len() as u32,
        lpData: map_name.as_ptr() as *mut c_void,
    };

    // Send WM_COPYDATA message to Pageant
    let result = unsafe {
        SendMessageA(
            hwnd,
            WM_COPYDATA,
            WPARAM::default(),
            LPARAM(&mut copy_data as *mut _ as isize),
        )
    };

    if result.0 == 0 {
        unsafe {
            let _ = UnmapViewOfFile(view);
            let _ = CloseHandle(handle);
        }
        return Err(Error::from_thread());
    }

    // Read response from shared memory
    let response_len = unsafe { (view.Value as *mut u32).read_unaligned() }.to_be() as usize;
    if response_len > 0 && response_len <= PUTTY_IPC_MAXLEN - 4 {
        let response =
            unsafe { std::slice::from_raw_parts((view.Value as *const u8).add(4), response_len) };
        println!("Received response: {}", String::from_utf8_lossy(response));
    }

    // Clean up
    unsafe {
        let _ = UnmapViewOfFile(view);
        let _ = CloseHandle(handle);
    }

    Ok(())
}
