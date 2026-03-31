use windows::core::{Error, Result, HRESULT};
use windows::Win32::Foundation::HANDLE;
use windows::Win32::System::Environment::CreateEnclave;

fn call_create_enclave() -> HRESULT {
    unsafe {
        let ptr = CreateEnclave(HANDLE::default(), None, 0, 0, 0, std::ptr::null(), 0, None);
        if ptr.is_null() {
            Error::from_thread().code()
        } else {
            HRESULT::from_win32(0)
        }
    }
}
