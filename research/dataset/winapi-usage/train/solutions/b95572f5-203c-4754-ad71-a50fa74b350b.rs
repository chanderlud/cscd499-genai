use windows::core::{Error, Result, HRESULT};
use windows::Win32::Networking::WinSock::{AcceptEx, SOCKET};

fn call_accept_ex() -> HRESULT {
    unsafe {
        let res = AcceptEx(
            SOCKET(0),
            SOCKET(0),
            std::ptr::null_mut(),
            0,
            0,
            0,
            std::ptr::null_mut(),
            std::ptr::null_mut(),
        );
        if res.0 != 0 {
            HRESULT::from_win32(0)
        } else {
            Error::from_thread().code()
        }
    }
}
