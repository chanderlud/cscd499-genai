use windows::core::Error;
use windows::Win32::Foundation::WIN32_ERROR;
use windows::Win32::Networking::WinSock::{WSACleanup, SOCKET_ERROR};

fn call_wsa_cleanup() -> WIN32_ERROR {
    let result = unsafe { WSACleanup() };
    if result == SOCKET_ERROR {
        let err = Error::from_thread();
        WIN32_ERROR::from_error(&err).unwrap_or_default()
    } else {
        WIN32_ERROR(0)
    }
}
