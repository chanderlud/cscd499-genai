use windows::Win32::Foundation::WIN32_ERROR;
use windows::Win32::System::Environment::CallEnclave;

fn call_call_enclave() -> WIN32_ERROR {
    unsafe {
        match CallEnclave(0, std::ptr::null(), false, std::ptr::null_mut()) {
            Ok(()) => WIN32_ERROR(0),
            Err(e) => WIN32_ERROR::from_error(&e).unwrap_or_default(),
        }
    }
}
