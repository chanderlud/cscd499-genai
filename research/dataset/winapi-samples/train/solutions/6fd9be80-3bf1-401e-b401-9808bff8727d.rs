use windows::core::Result;

pub fn ok_if_error_in_list<const N: usize>(
    result: Result<()>,
    error_codes: &[u32; N],
) -> Result<()> {
    match result {
        Ok(()) => Ok(()),
        Err(e) => {
            // Extract the HRESULT from the error
            let hresult = e.code();

            // Check if this is a Win32 error by examining the facility code
            // Win32 errors have facility code 7 (FACILITY_WIN32)
            // HRESULT format: bits 31-16 = facility, bits 15-0 = code
            let facility = (hresult.0 >> 16) & 0x7FFF;
            let code = hresult.0 & 0xFFFF;

            if facility == 7 {
                // This is a Win32 error, check if the code is in our list
                for &allowed_code in error_codes {
                    if code == allowed_code as i32 {
                        return Ok(());
                    }
                }
            }

            // Not in the list or not a Win32 error, return the original error
            Err(e)
        }
    }
}
