use windows::core::{Error, Result};
use windows::Win32::Foundation::HWND;
use windows::Win32::Graphics::Dwm::DwmAttachMilContent;

fn call_dwm_attach_mil_content() -> Result<Result<()>> {
    // SAFETY: Calling DwmAttachMilContent with a default HWND.
    // The API is unsafe due to raw handle usage, but passing a null/default handle is safe.
    let res = unsafe { DwmAttachMilContent(HWND::default()) };
    Ok(res)
}
