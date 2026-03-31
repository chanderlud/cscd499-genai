use windows::core::{Error, Result};
use windows::Win32::Graphics::Printing::{AddFormW, PRINTER_HANDLE};

fn call_add_form_w() -> Result<windows::core::BOOL> {
    // SAFETY: Calling AddFormW with null handle and null form pointer.
    // The API will fail, but we correctly capture the error via Error::from_thread().
    unsafe {
        let result = AddFormW(
            PRINTER_HANDLE {
                Value: std::ptr::null_mut(),
            },
            1,
            std::ptr::null(),
        );
        if result.as_bool() {
            Ok(result)
        } else {
            Err(Error::from_thread())
        }
    }
}
