use windows::core::{Error, Result};
use windows::Win32::Foundation::WIN32_ERROR;
use windows::Win32::Graphics::Printing::{AddJobA, PRINTER_HANDLE};

fn call_add_job_a() -> WIN32_ERROR {
    let mut needed = 0u32;
    let success = unsafe { AddJobA(PRINTER_HANDLE::default(), 1, None, &mut needed) };

    if success.as_bool() {
        WIN32_ERROR(0)
    } else {
        let err = Error::from_thread();
        WIN32_ERROR(err.code().0 as u32)
    }
}
