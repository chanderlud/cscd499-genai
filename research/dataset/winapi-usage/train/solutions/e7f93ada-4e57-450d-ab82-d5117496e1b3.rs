use windows::core::{Error, Result, HRESULT};
use windows::Win32::System::WinRT::CreateControlInput;

fn call_create_control_input() -> HRESULT {
    match unsafe { CreateControlInput::<windows::core::IUnknown>() } {
        Ok(_) => HRESULT::default(),
        Err(e) => e.code(),
    }
}
