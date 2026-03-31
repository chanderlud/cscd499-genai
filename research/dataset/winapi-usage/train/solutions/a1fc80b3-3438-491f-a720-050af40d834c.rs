use windows::core::{Error, Result};
use windows::Win32::Graphics::Printing::{AbortPrinter, PRINTER_HANDLE};

fn call_abort_printer() -> Result<windows::core::BOOL> {
    let handle = PRINTER_HANDLE::default();
    // SAFETY: Calling AbortPrinter with a null handle is safe; the API will return FALSE
    // and set the thread's last error code, which we capture and propagate below.
    let result = unsafe { AbortPrinter(handle) };
    result.ok()?;
    Ok(result)
}
