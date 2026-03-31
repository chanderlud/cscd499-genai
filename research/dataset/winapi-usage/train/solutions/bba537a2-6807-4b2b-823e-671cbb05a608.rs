use windows::core::HRESULT;
use windows::Win32::NetworkManagement::NetManagement::LogErrorW;

fn call_log_error_w() -> HRESULT {
    unsafe {
        LogErrorW(0, &[], 0);
        HRESULT(0)
    }
}
