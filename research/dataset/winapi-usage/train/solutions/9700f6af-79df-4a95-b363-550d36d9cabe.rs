use windows::core::{Error, Result, BOOL};
use windows::Win32::Graphics::Printing::{AddFormA, PRINTER_HANDLE};

fn call_add_form_a() -> Result<BOOL> {
    let result = unsafe {
        AddFormA(
            PRINTER_HANDLE {
                Value: std::ptr::null_mut(),
            },
            1,
            std::ptr::null(),
        )
    };
    result.ok()?;
    Ok(result)
}
