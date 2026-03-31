#![allow(dead_code)]

use windows::Win32::Foundation::WIN32_ERROR;
use windows::Win32::Graphics::Direct3D::Fxc::D3DCompile2;

fn call_d3_d_compile2() -> WIN32_ERROR {
    let mut ppcode = None;
    let mut pperrormsgs = None;
    // SAFETY: Passing null pointers and zero sizes is safe for this API call; it will fail gracefully.
    match unsafe {
        D3DCompile2(
            std::ptr::null(),
            0,
            None,
            None,
            None,
            None,
            None,
            0,
            0,
            0,
            None,
            0,
            &mut ppcode,
            Some(&mut pperrormsgs),
        )
    } {
        Ok(()) => WIN32_ERROR(0),
        Err(e) => WIN32_ERROR(e.code().0 as u32),
    }
}
