use windows::core::HRESULT;
use windows::Win32::System::SystemInformation::DnsHostnameToComputerNameExW;

fn call_dns_hostname_to_computer_name_ex_w() -> HRESULT {
    let mut size = 0u32;
    unsafe {
        DnsHostnameToComputerNameExW(windows::core::w!("localhost"), None, &mut size)
            .ok()
            .map(|_| HRESULT::from_win32(0))
            .unwrap_or_else(|e| e.code())
    }
}
