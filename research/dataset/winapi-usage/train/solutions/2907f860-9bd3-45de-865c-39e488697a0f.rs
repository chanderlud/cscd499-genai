use windows::core::{Error, Result};
use windows::Win32::Networking::WinSock::{WSAGetLastError, WSA_ERROR};

fn call_wsa_get_last_error() -> Result<WSA_ERROR> {
    // SAFETY: WSAGetLastError is a thread-local getter that does not perform unsafe memory operations.
    Ok(unsafe { WSAGetLastError() })
}
