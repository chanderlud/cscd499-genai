use windows::core::{Error, Result};
use windows::Win32::Foundation::HWND;
use windows::Win32::UI::Input::Ime::{ImmAssociateContextEx, HIMC};

fn call_imm_associate_context_ex() -> Result<windows::core::BOOL> {
    let hwnd = HWND::default();
    let himc = HIMC::default();
    let flags = 0u32;

    let result = unsafe { ImmAssociateContextEx(hwnd, himc, flags) };
    if result.as_bool() {
        Ok(result)
    } else {
        Err(Error::from_thread())
    }
}
