#![allow(unused_imports)]
use windows::core::{Error, Result};
use windows::Win32::Foundation::{HWND, WIN32_ERROR};
use windows::Win32::UI::Input::Ime::{ImmAssociateContextEx, HIMC};

fn call_imm_associate_context_ex() -> WIN32_ERROR {
    let result = unsafe { ImmAssociateContextEx(HWND::default(), HIMC::default(), 0) };
    if result.as_bool() {
        WIN32_ERROR(0)
    } else {
        let err = Error::from_thread();
        WIN32_ERROR(err.code().0 as u32)
    }
}
