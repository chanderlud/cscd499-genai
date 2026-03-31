use windows::core::{Error, Result};
use windows::Win32::Foundation::HWND;
use windows::Win32::Graphics::Dwm::DwmDetachMilContent;

fn call_dwm_detach_mil_content() -> Result<windows::core::Result<()>> {
    let hwnd = HWND::default();
    // SAFETY: Passing a default HWND is safe; the API handles invalid handles gracefully
    // by returning an error, which is properly captured in the Result.
    let result = unsafe { DwmDetachMilContent(hwnd) };
    Ok(result)
}
