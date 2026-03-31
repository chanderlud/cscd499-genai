use windows::core::Error;
use windows::Win32::Foundation::WIN32_ERROR;
use windows::Win32::Graphics::Printing::{AddFormA, PRINTER_HANDLE};

fn call_add_form_a() -> WIN32_ERROR {
    unsafe {
        let result = AddFormA(
            PRINTER_HANDLE {
                Value: std::ptr::null_mut(),
            },
            1,
            std::ptr::null(),
        );
        if result.as_bool() {
            WIN32_ERROR(0)
        } else {
            WIN32_ERROR(Error::from_thread().code().0 as u32)
        }
    }
}
