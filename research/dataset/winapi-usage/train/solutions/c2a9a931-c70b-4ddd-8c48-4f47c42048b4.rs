use windows::core::Result;
use windows::Win32::Foundation::WIN32_ERROR;
use windows::Win32::NetworkManagement::NetManagement::NetApiBufferFree;

fn call_net_api_buffer_free() -> windows::Win32::Foundation::WIN32_ERROR {
    let result = unsafe { NetApiBufferFree(None) };
    WIN32_ERROR(result)
}
