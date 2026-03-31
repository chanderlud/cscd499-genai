use windows::core::{Error, HRESULT};
use windows::Win32::Foundation::S_OK;
use windows::Win32::Graphics::Printing::{AddJobA, PRINTER_HANDLE};

fn call_add_job_a() -> HRESULT {
    let mut pcbneeded = 0u32;
    // SAFETY: We pass a null printer handle and a valid mutable pointer for pcbneeded.
    // The function is unsafe as it interacts directly with the Win32 printing API.
    let success = unsafe {
        AddJobA(
            PRINTER_HANDLE {
                Value: std::ptr::null_mut(),
            },
            1,
            None,
            &mut pcbneeded,
        )
    };

    if success.as_bool() {
        S_OK
    } else {
        Error::from_thread().code()
    }
}
