use windows::core::{Error, Result};
use windows::Win32::Foundation::{HWND, WIN32_ERROR};
use windows::Win32::UI::Input::Ime::{ImmAssociateContext, HIMC};

fn call_imm_associate_context() -> WIN32_ERROR {
    unsafe {
        ImmAssociateContext(HWND::default(), HIMC::default());
    }
    WIN32_ERROR(0)
}
