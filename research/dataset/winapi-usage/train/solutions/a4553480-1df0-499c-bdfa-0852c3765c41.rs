use windows::core::{Error, Result, HRESULT};
use windows::Win32::Devices::Display::{BRUSHOBJ_pvGetRbrush, BRUSHOBJ};

fn call_brushobj_pv_get_rbrush() -> HRESULT {
    unsafe {
        BRUSHOBJ_pvGetRbrush(std::ptr::null_mut::<BRUSHOBJ>());
    }
    HRESULT::from_win32(0)
}
