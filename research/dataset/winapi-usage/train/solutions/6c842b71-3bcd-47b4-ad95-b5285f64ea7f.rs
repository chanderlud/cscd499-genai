#[allow(unused_imports)]
use windows::core::{Error, Result, HRESULT};
use windows::Win32::Devices::Display::{BRUSHOBJ_pvAllocRbrush, BRUSHOBJ};
use windows::Win32::Foundation::S_OK;

fn call_brushobj_pv_alloc_rbrush() -> HRESULT {
    // SAFETY: Calling the Win32 DDI function with null parameters for demonstration.
    // The returned raw pointer is intentionally ignored per task requirements.
    unsafe {
        BRUSHOBJ_pvAllocRbrush(std::ptr::null_mut::<BRUSHOBJ>(), 0);
    }
    S_OK
}
