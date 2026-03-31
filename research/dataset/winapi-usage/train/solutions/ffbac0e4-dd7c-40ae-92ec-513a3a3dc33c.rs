use windows::core::HRESULT;
use windows::Win32::NetworkManagement::NetManagement::LogErrorA;

fn call_log_error_a() -> HRESULT {
    unsafe {
        LogErrorA(0, &[], 0);
        HRESULT(0)
    }
}
