use windows::core::HRESULT;
use windows::Win32::NetworkManagement::IpHelper::FreeMibTable;

fn call_free_mib_table() -> windows::core::HRESULT {
    unsafe {
        FreeMibTable(std::ptr::null());
    }
    HRESULT(0)
}
