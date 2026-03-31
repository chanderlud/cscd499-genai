use windows::core::{Error, Result, HRESULT};
use windows::Win32::Networking::WinSock::WSACleanup;

fn call_wsa_cleanup() -> HRESULT {
    let res = unsafe { WSACleanup() };
    if res == 0 {
        HRESULT(0)
    } else {
        Error::from_thread().code()
    }
}
