use windows::core::HRESULT;
use windows::Win32::NetworkManagement::IpHelper::{
    CancelIfTimestampConfigChange, HIFTIMESTAMPCHANGE,
};

fn call_cancel_if_timestamp_config_change() -> HRESULT {
    unsafe {
        CancelIfTimestampConfigChange(HIFTIMESTAMPCHANGE(std::ptr::null_mut()));
        HRESULT::default()
    }
}
