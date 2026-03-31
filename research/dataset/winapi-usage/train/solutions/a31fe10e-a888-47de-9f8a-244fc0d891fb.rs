use windows::core::IUnknown;
use windows::Win32::Foundation::WIN32_ERROR;
use windows::Win32::System::WinRT::CreateControlInput;

#[allow(dead_code)]
fn call_create_control_input() -> WIN32_ERROR {
    match unsafe { CreateControlInput::<IUnknown>() } {
        Ok(_) => WIN32_ERROR(0),
        Err(e) => WIN32_ERROR(e.code().0 as u32),
    }
}
