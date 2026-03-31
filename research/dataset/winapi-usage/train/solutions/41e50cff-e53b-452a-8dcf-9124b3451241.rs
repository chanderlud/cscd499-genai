use windows::Win32::Foundation::WIN32_ERROR;
use windows::Win32::System::Ole::BstrFromVector;

fn call_bstr_from_vector() -> WIN32_ERROR {
    let result = unsafe { BstrFromVector(std::ptr::null()) };
    match result {
        Ok(_) => WIN32_ERROR(0),
        Err(e) => WIN32_ERROR(e.code().0 as u32),
    }
}
