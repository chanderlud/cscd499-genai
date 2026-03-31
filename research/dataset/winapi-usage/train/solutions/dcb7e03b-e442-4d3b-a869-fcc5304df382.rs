use windows::core::{Error, Result, HRESULT};
use windows::Win32::System::Ole::CreateDispTypeInfo;

fn call_create_disp_type_info() -> HRESULT {
    match unsafe { CreateDispTypeInfo(std::ptr::null_mut(), 0, std::ptr::null_mut()) } {
        Ok(()) => HRESULT(0),
        Err(e) => e.code(),
    }
}
