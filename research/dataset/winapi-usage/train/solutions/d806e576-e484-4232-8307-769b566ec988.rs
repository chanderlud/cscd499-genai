use windows::Win32::Foundation::S_OK;
use windows::Win32::System::Services::{CloseServiceHandle, SC_HANDLE};

fn call_close_service_handle() -> windows::core::HRESULT {
    let hscobject = SC_HANDLE(std::ptr::null_mut());

    unsafe {
        match CloseServiceHandle(hscobject) {
            Ok(()) => S_OK,
            Err(e) => e.code(),
        }
    }
}
