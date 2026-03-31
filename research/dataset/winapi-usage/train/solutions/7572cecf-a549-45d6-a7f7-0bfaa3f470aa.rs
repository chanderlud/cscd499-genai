use windows::Win32::Foundation::WIN32_ERROR;
use windows::Win32::NetworkManagement::NetManagement::LogErrorW;

fn call_log_error_w() -> WIN32_ERROR {
    unsafe { LogErrorW(0, &[], 0) };
    WIN32_ERROR(0)
}
