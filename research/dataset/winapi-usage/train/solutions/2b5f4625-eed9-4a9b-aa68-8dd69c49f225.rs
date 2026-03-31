use windows::Win32::Foundation::{GetLastError, WIN32_ERROR};
use windows::Win32::Globalization::CompareStringA;

fn call_compare_string_a() -> WIN32_ERROR {
    let result = unsafe { CompareStringA(0, 0, &[], &[]) };
    if result.0 == 0 {
        unsafe { GetLastError() }
    } else {
        WIN32_ERROR(0)
    }
}
