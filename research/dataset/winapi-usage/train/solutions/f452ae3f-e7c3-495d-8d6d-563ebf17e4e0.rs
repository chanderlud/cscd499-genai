use windows::core::{Error, Result, BOOL};
use windows::Win32::Foundation::HANDLE;
use windows::Win32::Security::{AccessCheck, PSECURITY_DESCRIPTOR};

fn call_access_check() -> Result<()> {
    let mut privilegesetlength = 0u32;
    let mut grantedaccess = 0u32;
    let mut accessstatus = BOOL(0);

    unsafe {
        AccessCheck(
            PSECURITY_DESCRIPTOR(std::ptr::null_mut()),
            HANDLE(std::ptr::null_mut()),
            0,
            std::ptr::null(),
            None,
            &mut privilegesetlength,
            &mut grantedaccess,
            &mut accessstatus,
        )?;
    }
    Ok(())
}
