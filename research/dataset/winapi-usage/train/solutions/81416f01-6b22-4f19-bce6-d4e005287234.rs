use windows::core::BOOL;
use windows::core::{Error, Result};
use windows::Win32::Graphics::Printing::{AddJobA, PRINTER_HANDLE};

fn call_add_job_a() -> Result<BOOL> {
    let mut needed = 0u32;
    // SAFETY: Calling AddJobA with a null printer handle and valid pointers.
    // The call will fail, but we handle the error via result.ok()?.
    let result = unsafe {
        AddJobA(
            PRINTER_HANDLE {
                Value: std::ptr::null_mut(),
            },
            1,
            None,
            &mut needed,
        )
    };

    result.ok()?;
    Ok(result)
}
