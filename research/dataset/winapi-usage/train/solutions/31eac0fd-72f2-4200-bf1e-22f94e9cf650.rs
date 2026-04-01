use windows::Win32::Foundation::WIN32_ERROR;
use windows::Win32::System::RestartManager::RmGetList;

fn call_rm_get_list() -> windows::Win32::Foundation::WIN32_ERROR {
    unsafe {
        RmGetList(
            0,
            std::ptr::null_mut(),
            std::ptr::null_mut(),
            None,
            std::ptr::null_mut(),
        )
    }
}
