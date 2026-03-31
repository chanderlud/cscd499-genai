use windows::core::{Error, Result};
use windows::Win32::Foundation::HWND;
use windows::Win32::UI::Input::Ime::{ImmAssociateContext, HIMC};

fn call_imm_associate_context() -> Result<HIMC> {
    // SAFETY: Passing null handles to ImmAssociateContext is safe and valid for disassociation.
    let result = unsafe { ImmAssociateContext(HWND::default(), HIMC::default()) };
    Ok(result)
}
