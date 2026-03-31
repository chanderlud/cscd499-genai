use windows::Win32::Foundation::WIN32_ERROR;
use windows::Win32::NetworkManagement::IpHelper::{
    CancelIfTimestampConfigChange, HIFTIMESTAMPCHANGE,
};

fn call_cancel_if_timestamp_config_change() -> WIN32_ERROR {
    unsafe { CancelIfTimestampConfigChange(HIFTIMESTAMPCHANGE(std::ptr::null_mut())) };
    WIN32_ERROR(0)
}
