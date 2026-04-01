use windows::core::{Error, Result};
use windows::Win32::NetworkManagement::NetManagement::NetApiBufferFree;

fn call_net_api_buffer_free() -> windows::core::Result<u32> {
    // Call NetApiBufferFree with NULL (no buffer to free)
    let result = unsafe { NetApiBufferFree(None) };

    // Check if the call succeeded (0 = success)
    if result == 0 {
        Ok(result)
    } else {
        // Convert the WIN32_ERROR code to HRESULT, then to Error
        let hresult = windows::core::HRESULT::from_win32(result);
        Err(Error::from_hresult(hresult))
    }
}
