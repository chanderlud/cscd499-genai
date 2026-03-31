use windows::core::{Error, Result};
use windows::Win32::Networking::WinSock::WSACleanup;

#[allow(dead_code)]
fn call_wsa_cleanup() -> Result<i32> {
    // SAFETY: WSACleanup is a standard Winsock API that safely cleans up resources.
    let result = unsafe { WSACleanup() };
    if result == -1 {
        Err(Error::from_thread())
    } else {
        Ok(result)
    }
}
