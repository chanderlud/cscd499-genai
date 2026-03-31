use windows::core::{Error, Result};
use windows::Win32::Devices::Display::BRUSHOBJ_pvGetRbrush;
use windows::Win32::Foundation::WIN32_ERROR;

fn call_brushobj_pv_get_rbrush() -> WIN32_ERROR {
    unsafe {
        let _ = BRUSHOBJ_pvGetRbrush(std::ptr::null_mut());
    }
    WIN32_ERROR(0)
}
