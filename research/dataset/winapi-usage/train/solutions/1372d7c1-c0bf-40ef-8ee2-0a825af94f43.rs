#[allow(unused_imports)]
use windows::core::{Error, Result, HRESULT};
use windows::Win32::Foundation::{CompareObjectHandles, HANDLE};

fn call_compare_object_handles() -> HRESULT {
    let result = unsafe { CompareObjectHandles(HANDLE::default(), HANDLE::default()) };
    if result.as_bool() {
        HRESULT::default()
    } else {
        Error::from_thread().code()
    }
}
