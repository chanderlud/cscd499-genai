use windows::Win32::Foundation::{GetLastError, WIN32_ERROR};
use windows::Win32::System::SystemInformation::DnsHostnameToComputerNameExW;

fn call_dns_hostname_to_computer_name_ex_w() -> WIN32_ERROR {
    let mut size = 0u32;
    let result =
        unsafe { DnsHostnameToComputerNameExW(windows::core::w!("localhost"), None, &mut size) };

    if result.as_bool() {
        WIN32_ERROR(0)
    } else {
        unsafe { GetLastError() }
    }
}
