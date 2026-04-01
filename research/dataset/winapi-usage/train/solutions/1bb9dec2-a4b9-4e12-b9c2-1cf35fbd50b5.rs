use windows::core::{Error, Result};
use windows::Win32::NetworkManagement::IpHelper::FreeMibTable;

fn call_free_mib_table() -> windows::core::Result<()> {
    unsafe {
        // FreeMibTable takes a pointer to MIB table memory and frees it
        // Passing null pointer is a valid concrete parameter value
        FreeMibTable(std::ptr::null());
    }
    Ok(())
}
