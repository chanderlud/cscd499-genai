use windows::Win32::Foundation::HANDLE;
use windows::Win32::System::IO::{GetOverlappedResult, OVERLAPPED};

fn call_get_overlapped_result() -> windows::core::Result<()> {
    // Create a null handle (will fail, but demonstrates API call pattern)
    let hfile = HANDLE(std::ptr::null_mut());

    // Initialize OVERLAPPED structure with zero values using Default
    let mut overlapped = OVERLAPPED::default();

    // Variable to receive bytes transferred
    let mut bytes_transferred: u32 = 0;

    // Call GetOverlappedResult (unsafe Win32 API)
    // GetOverlappedResult returns windows_core::Result<()> directly
    unsafe { GetOverlappedResult(hfile, &overlapped, &mut bytes_transferred, true) }
}
