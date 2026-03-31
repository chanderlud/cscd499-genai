use windows::core::Error;
use windows::Win32::Foundation::WIN32_ERROR;
use windows::Win32::Graphics::Printing::{AddFormW, PRINTER_HANDLE};

fn call_add_form_w() -> WIN32_ERROR {
    unsafe {
        let result = AddFormW(
            PRINTER_HANDLE {
                Value: std::ptr::null_mut(),
            },
            1,
            std::ptr::null(),
        );
        if result.0 != 0 {
            WIN32_ERROR(0)
        } else {
            let err = Error::from_thread();
            WIN32_ERROR(err.code().0 as u32)
        }
    }
}
