use windows::core::{Result, BOOL};
use windows::Win32::Graphics::Printing::{AddJobW, PRINTER_HANDLE};

fn call_add_job_w() -> Result<BOOL> {
    unsafe {
        let mut pcbneeded: u32 = 0;
        // PRINTER_HANDLE is a struct with a named Value field
        let hprinter = PRINTER_HANDLE {
            Value: std::ptr::null_mut(),
        };
        let level: u32 = 1;

        // AddJobW returns BOOL directly, wrap in Ok() to match Result<BOOL> return type
        Ok(AddJobW(hprinter, level, None, &mut pcbneeded))
    }
}
