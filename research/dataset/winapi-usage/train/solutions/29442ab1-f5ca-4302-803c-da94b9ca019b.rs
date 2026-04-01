use windows::core::{Error, Result};
use windows::Win32::Foundation::{ERROR_SUCCESS, WIN32_ERROR};
use windows::Win32::System::Ole::CreateOleAdviseHolder;

fn call_create_ole_advise_holder() -> WIN32_ERROR {
    match unsafe { CreateOleAdviseHolder() } {
        Ok(_) => ERROR_SUCCESS,
        Err(err) => WIN32_ERROR(err.code().0 as u32),
    }
}
