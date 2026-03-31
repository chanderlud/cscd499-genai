use windows::Win32::Foundation::{GetLastError, WIN32_ERROR};
use windows::Win32::Graphics::Printing::{AbortPrinter, PRINTER_HANDLE};

fn call_abort_printer() -> WIN32_ERROR {
    unsafe {
        let result = AbortPrinter(PRINTER_HANDLE {
            Value: std::ptr::null_mut(),
        });
        if result.as_bool() {
            WIN32_ERROR(0)
        } else {
            GetLastError()
        }
    }
}
