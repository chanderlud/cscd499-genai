use windows::core::{Error, Result};
use windows::Win32::System::SystemInformation::DnsHostnameToComputerNameExW;

fn call_dns_hostname_to_computer_name_ex_w() -> Result<()> {
    let mut name = [0u16; 256];
    let mut size = name.len() as u32;

    // SAFETY: We provide a valid null-terminated PCWSTR for hostname, a valid mutable PWSTR
    // pointing to a sufficiently sized buffer for computername, and a valid pointer to u32 for nsize.
    let success = unsafe {
        DnsHostnameToComputerNameExW(
            windows::core::w!("localhost"),
            Some(windows::core::PWSTR(name.as_mut_ptr())),
            &mut size,
        )
    };

    if success.as_bool() {
        Ok(())
    } else {
        Err(Error::from_thread())
    }
}
