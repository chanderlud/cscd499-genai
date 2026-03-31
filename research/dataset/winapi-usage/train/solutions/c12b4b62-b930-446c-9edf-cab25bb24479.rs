#![allow(dead_code, unused_imports)]

use windows::core::{Error, Result, HRESULT};
use windows::Win32::Graphics::Printing::{AbortPrinter, PRINTER_HANDLE};

fn call_abort_printer() -> HRESULT {
    // SAFETY: Passing a null handle is safe for this exercise; the API will fail gracefully
    // and set the thread's last error code, which we capture below.
    let success = unsafe {
        AbortPrinter(PRINTER_HANDLE {
            Value: std::ptr::null_mut(),
        })
    };
    if success.as_bool() {
        HRESULT(0)
    } else {
        Error::from_thread().code()
    }
}
