use windows::core::{Error, HRESULT};
use windows::Win32::Globalization::CompareStringA;

fn call_compare_string_a() -> HRESULT {
    let s1 = b"abc";
    let s2 = b"abc";

    // SAFETY: We pass valid byte slices cast to i8 slices as required by the API signature.
    unsafe {
        let lp1 = std::slice::from_raw_parts(s1.as_ptr() as *const i8, s1.len());
        let lp2 = std::slice::from_raw_parts(s2.as_ptr() as *const i8, s2.len());

        let result = CompareStringA(0, 0, lp1, lp2);

        if result.0 == 0 {
            Error::from_thread().code()
        } else {
            HRESULT::from_win32(0)
        }
    }
}
