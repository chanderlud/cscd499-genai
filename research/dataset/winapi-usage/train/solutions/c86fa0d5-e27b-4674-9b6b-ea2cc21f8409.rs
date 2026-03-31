use windows::Win32::System::Com::CUSTDATA;
use windows::Win32::System::Ole::ClearCustData;

fn call_clear_cust_data() -> windows::Win32::Foundation::WIN32_ERROR {
    let mut cust_data = CUSTDATA {
        cCustData: 0,
        prgCustData: std::ptr::null_mut(),
    };
    // SAFETY: We pass a valid mutable pointer to a properly initialized CUSTDATA struct.
    // ClearCustData safely handles zeroed data without accessing invalid memory.
    unsafe {
        ClearCustData(&mut cust_data);
    }
    windows::Win32::Foundation::WIN32_ERROR(0)
}
