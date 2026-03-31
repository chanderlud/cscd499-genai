use windows::core::Error;
use windows::Win32::Graphics::Printing::{AddFormA, PRINTER_HANDLE};

fn call_add_form_a() -> windows::core::HRESULT {
    unsafe {
        if AddFormA(
            PRINTER_HANDLE {
                Value: std::ptr::null_mut(),
            },
            0,
            std::ptr::null(),
        )
        .as_bool()
        {
            windows::core::HRESULT(0)
        } else {
            Error::from_thread().code()
        }
    }
}
