use windows::core::Error;
use windows::Win32::Foundation::WIN32_ERROR;
use windows::Win32::Graphics::Printing::{AddJobW, PRINTER_HANDLE};

fn call_add_job_w() -> windows::Win32::Foundation::WIN32_ERROR {
    let hprinter = PRINTER_HANDLE {
        Value: std::ptr::null_mut(),
    };
    let level: u32 = 1;
    let pdata: Option<&mut [u8]> = None;
    let mut pcbneeded: u32 = 0;

    let result = unsafe { AddJobW(hprinter, level, pdata, &mut pcbneeded) };

    if result.as_bool() {
        WIN32_ERROR(0)
    } else {
        WIN32_ERROR(Error::from_thread().code().0 as u32)
    }
}
