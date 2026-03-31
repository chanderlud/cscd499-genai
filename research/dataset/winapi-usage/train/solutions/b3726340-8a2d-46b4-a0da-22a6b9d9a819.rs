use windows::core::Error;
use windows::Win32::Foundation::WIN32_ERROR;
use windows::Win32::Graphics::Gdi::AddFontMemResourceEx;

fn call_add_font_mem_resource_ex() -> WIN32_ERROR {
    let handle = unsafe { AddFontMemResourceEx(std::ptr::null(), 0, None, std::ptr::null()) };
    if handle.0.is_null() {
        WIN32_ERROR(Error::from_thread().code().0 as u32)
    } else {
        WIN32_ERROR(0)
    }
}
