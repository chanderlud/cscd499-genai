use windows::core::{Error, Result, HRESULT, PCWSTR};
use windows::Win32::Foundation::STATUS_SUCCESS;
use windows::Win32::Security::Cryptography::{
    BCryptCloseAlgorithmProvider, BCryptGetFipsAlgorithmMode, BCryptOpenAlgorithmProvider,
    BCRYPT_ALG_HANDLE, BCRYPT_OPEN_ALGORITHM_PROVIDER_FLAGS,
};

// Define the constant locally since it's not in the windows crate
const BCRYPT_FIPS_ALGORITHM_FLAG: u32 = 0x00000002;

fn wide_null(s: &str) -> Vec<u16> {
    use std::iter::once;
    s.encode_utf16().chain(once(0)).collect()
}

fn check_fips_algorithm_support(algorithm_id: &str) -> Result<(bool, bool)> {
    // Check system FIPS mode status
    let mut fips_enabled: u8 = 0;
    // SAFETY: BCryptGetFipsAlgorithmMode is a valid CNG API call
    let status = unsafe { BCryptGetFipsAlgorithmMode(&mut fips_enabled) };
    if status != STATUS_SUCCESS {
        return Err(Error::from_hresult(HRESULT::from_win32(status.0 as u32)));
    }
    let fips_mode_enabled = fips_enabled != 0;

    // Check if algorithm is FIPS-approved
    let alg_id_wide = wide_null(algorithm_id);
    let mut alg_handle = BCRYPT_ALG_HANDLE::default();

    // SAFETY: BCryptOpenAlgorithmProvider is a valid CNG API call
    let status = unsafe {
        BCryptOpenAlgorithmProvider(
            &mut alg_handle,
            PCWSTR(alg_id_wide.as_ptr()),
            None,
            BCRYPT_OPEN_ALGORITHM_PROVIDER_FLAGS(BCRYPT_FIPS_ALGORITHM_FLAG),
        )
    };

    let algorithm_is_fips_approved = if status == STATUS_SUCCESS {
        // SAFETY: We have a valid algorithm handle to close
        unsafe { BCryptCloseAlgorithmProvider(alg_handle, 0) };
        true
    } else {
        // STATUS_INVALID_PARAMETER indicates algorithm not FIPS-approved
        // Other errors indicate actual failures
        let hresult = HRESULT::from_win32(status.0 as u32);
        if hresult == HRESULT::from_win32(0x80070057) {
            // ERROR_INVALID_PARAMETER
            false
        } else {
            return Err(Error::from_hresult(hresult));
        }
    };

    Ok((fips_mode_enabled, algorithm_is_fips_approved))
}
