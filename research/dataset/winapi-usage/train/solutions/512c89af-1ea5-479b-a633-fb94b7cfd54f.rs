use windows::core::Result;
use windows::Win32::NetworkManagement::IpHelper::{
    CancelIfTimestampConfigChange, HIFTIMESTAMPCHANGE,
};

fn call_cancel_if_timestamp_config_change() -> Result<()> {
    unsafe { CancelIfTimestampConfigChange(HIFTIMESTAMPCHANGE(std::ptr::null_mut())) };
    Ok(())
}
