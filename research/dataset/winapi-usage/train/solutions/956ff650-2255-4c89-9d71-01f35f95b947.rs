use windows::core::{Error, Result, HRESULT};
use windows::Win32::Graphics::Printing::{AddFormW, PRINTER_HANDLE};

fn call_add_form_w() -> HRESULT {
    // SAFETY: We pass concrete null values for this exercise. The call will fail and set
    // GetLastError, which we capture correctly via Error::from_thread().
    unsafe {
        let result = AddFormW(
            PRINTER_HANDLE {
                Value: std::ptr::null_mut(),
            },
            1,
            std::ptr::null::<u8>(),
        );
        if result.as_bool() {
            HRESULT(0)
        } else {
            Error::from_thread().code()
        }
    }
}
