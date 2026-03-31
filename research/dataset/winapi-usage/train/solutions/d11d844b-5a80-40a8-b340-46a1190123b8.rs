use windows::core::{Error, Result, HRESULT};
use windows::Win32::System::Diagnostics::Debug::AddVectoredContinueHandler;

fn call_add_vectored_continue_handler() -> HRESULT {
    unsafe {
        AddVectoredContinueHandler(0, None);
        HRESULT::from_win32(0)
    }
}
