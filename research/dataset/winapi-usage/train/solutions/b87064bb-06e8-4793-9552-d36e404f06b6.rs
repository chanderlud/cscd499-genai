use windows::Win32::System::Services::{CloseServiceHandle, SC_HANDLE};

fn call_close_service_handle() -> windows::core::Result<windows::core::Result<()>> {
    let hscobject = SC_HANDLE(std::ptr::null_mut());
    let result = unsafe { CloseServiceHandle(hscobject) };
    Ok(result)
}
