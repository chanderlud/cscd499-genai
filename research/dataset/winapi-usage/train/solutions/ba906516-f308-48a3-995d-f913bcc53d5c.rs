use windows::core::HRESULT;
use windows::core::{Error, Result};
use windows::Win32::Foundation::HWND;
use windows::Win32::UI::Input::Ime::{ImmAssociateContext, HIMC};

fn call_imm_associate_context() -> HRESULT {
    unsafe {
        ImmAssociateContext(HWND::default(), HIMC::default());
        HRESULT::from_win32(0)
    }
}
