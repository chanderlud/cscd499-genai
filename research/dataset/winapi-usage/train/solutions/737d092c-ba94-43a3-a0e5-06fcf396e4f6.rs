use windows::core::HRESULT;
use windows::core::{Error, Result};
use windows::Win32::NetworkManagement::NetManagement::NetApiBufferFree;

fn call_net_api_buffer_free() -> windows::core::HRESULT {
    // Call NetApiBufferFree with None (null pointer) as concrete parameter
    let result = unsafe { NetApiBufferFree(None) };

    // Convert the u32 return value (Win32 error code) to HRESULT
    HRESULT::from_win32(result)
}
