use windows::core::{Error, Result};
use windows::Win32::Foundation::WIN32_ERROR;
use windows::Win32::Networking::WinSock::WSAGetLastError;

fn call_wsa_get_last_error() -> WIN32_ERROR {
    // SAFETY: WSAGetLastError is a thread-local state query with no side effects or invalid memory access.
    WIN32_ERROR(unsafe { WSAGetLastError() }.0 as u32)
}
