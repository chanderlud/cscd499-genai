use windows::core::HRESULT;
#[allow(unused_imports)]
use windows::core::{Error, Result};
use windows::Win32::Foundation::{HWND, S_OK};
use windows::Win32::UI::Input::Ime::{ImmAssociateContextEx, HIMC};

fn call_imm_associate_context_ex() -> HRESULT {
    // SAFETY: Calling with default/null handles and zero flags is safe for this API.
    let result = unsafe { ImmAssociateContextEx(HWND::default(), HIMC::default(), 0) };
    if result.as_bool() {
        S_OK
    } else {
        Error::from_thread().code()
    }
}
