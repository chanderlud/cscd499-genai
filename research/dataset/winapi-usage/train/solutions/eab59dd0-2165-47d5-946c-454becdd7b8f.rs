use windows::core::{Error, Result};
use windows::Win32::Foundation::S_OK;
use windows::Win32::System::Threading::CreateEventA;

fn call_create_event_a() -> windows::core::HRESULT {
    let result = unsafe { CreateEventA(None, false, false, None) };

    match result {
        Ok(_) => S_OK,
        Err(e) => e.code(),
    }
}
