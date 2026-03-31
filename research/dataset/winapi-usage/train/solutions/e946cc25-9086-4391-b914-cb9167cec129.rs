use windows::Win32::Foundation::WIN32_ERROR;
use windows::Win32::System::Ole::CreateDispTypeInfo;

fn call_create_disp_type_info() -> WIN32_ERROR {
    unsafe {
        match CreateDispTypeInfo(std::ptr::null_mut(), 0, std::ptr::null_mut()) {
            Ok(()) => WIN32_ERROR(0),
            Err(e) => WIN32_ERROR(e.code().0 as u32),
        }
    }
}
