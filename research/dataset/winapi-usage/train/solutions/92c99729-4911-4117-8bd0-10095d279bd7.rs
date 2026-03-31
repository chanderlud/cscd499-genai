use windows::core::{Error, HRESULT};
use windows::Win32::Foundation::S_OK;
use windows::Win32::System::Power::DeletePwrScheme;

fn call_delete_pwr_scheme() -> HRESULT {
    // SAFETY: DeletePwrScheme is a standard Win32 API. We pass a concrete scheme ID.
    let success = unsafe { DeletePwrScheme(1) };
    if success {
        S_OK
    } else {
        Error::from_thread().code()
    }
}
