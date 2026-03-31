use windows::core::{Error, Result, HRESULT, PCWSTR};
use windows::Win32::NetworkManagement::NetManagement::I_NetLogonControl2;

fn call_i__net_logon_control2() -> Result<u32> {
    let mut buffer: *mut u8 = std::ptr::null_mut();

    // SAFETY: Calling I_NetLogonControl2 with null servername (local computer),
    // valid function/query codes, and a valid pointer for the output buffer.
    let result_code = unsafe {
        I_NetLogonControl2(
            PCWSTR::null(),
            1,
            1,
            std::ptr::null(),
            std::ptr::addr_of_mut!(buffer),
        )
    };

    if result_code != 0 {
        let hresult = HRESULT::from_win32(result_code);
        return Err(Error::from_hresult(hresult));
    }
    Ok(result_code)
}
