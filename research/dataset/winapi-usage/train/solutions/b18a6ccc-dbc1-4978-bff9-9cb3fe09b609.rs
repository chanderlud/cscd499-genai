use windows::core::{Error, Result};
use windows::Win32::Graphics::Printing::{AddJobW, PRINTER_HANDLE};

fn call_add_job_w() -> Result<()> {
    // Create a null printer handle for demonstration
    let hprinter = PRINTER_HANDLE {
        Value: std::ptr::null_mut(),
    };

    // Level 1 is a common level for AddJob
    let level: u32 = 1;

    // Empty buffer for data
    let mut data: Vec<u8> = Vec::new();

    // Pointer to get needed size
    let mut pcbneeded: u32 = 0;

    unsafe {
        let result = AddJobW(hprinter, level, Some(&mut data), &mut pcbneeded);

        if result.0 == 0 {
            // Failed - get error from thread
            return Err(Error::from_thread());
        } else {
            // Success
            Ok(())
        }
    }
}
