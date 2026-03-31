use windows::core::{Error, Result, HRESULT};
use windows::Win32::Networking::WinSock::WSAGetLastError;

fn call_wsa_get_last_error() -> HRESULT {
    // SAFETY: WSAGetLastError is a standard Winsock function that safely retrieves the last error.
    let err = unsafe { WSAGetLastError() };
    HRESULT(err.0)
}
