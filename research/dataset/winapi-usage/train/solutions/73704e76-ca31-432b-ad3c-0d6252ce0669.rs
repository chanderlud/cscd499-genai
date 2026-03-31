use windows::core::HRESULT;
use windows::Win32::System::Com::CUSTDATA;
use windows::Win32::System::Ole::ClearCustData;

fn call_clear_cust_data() -> HRESULT {
    let mut cust_data = CUSTDATA {
        cCustData: 0,
        prgCustData: std::ptr::null_mut(),
    };
    unsafe {
        ClearCustData(&mut cust_data);
        HRESULT(0)
    }
}
