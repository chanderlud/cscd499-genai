use windows::core::{Error, Result, BOOL};
use windows::Win32::Foundation::{HWND, LPARAM, LRESULT, WPARAM};
use windows::Win32::Graphics::Dwm::DwmDefWindowProc;

#[allow(dead_code)]
fn call_dwm_def_window_proc() -> Result<BOOL> {
    let mut lresult = LRESULT(0);
    // SAFETY: Passing default/null values to DwmDefWindowProc is safe; it handles invalid HWNDs gracefully.
    let res = unsafe { DwmDefWindowProc(HWND::default(), 0, WPARAM(0), LPARAM(0), &mut lresult) };
    Ok(res)
}
