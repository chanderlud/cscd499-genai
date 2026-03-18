use std::ffi::OsStr;
use windows::core::{Error, Result, HRESULT, PCWSTR};
use windows::Win32::Foundation::NTSTATUS;
use windows::Win32::Security::Cryptography::{
    BCryptCloseAlgorithmProvider, BCryptGetFipsAlgorithmMode, BCryptGetProperty,
    BCryptOpenAlgorithmProvider, BCRYPT_ALG_HANDLE, BCRYPT_ALG_HANDLE_HMAC_FLAG,
    BCRYPT_OPEN_ALGORITHM_PROVIDER_FLAGS,
};

fn wide_null(s: &OsStr) -> Vec<u16> {
    use std::{iter::once, os::windows::ffi::OsStrExt};
    s.encode_wide().chain(once(0)).collect()
}

fn is_algorithm_fips_approved(algorithm_name: &str) -> Result<bool> {
    // Step 1: Check if system is in FIPS mode
    let mut fips_enabled = 0u8;
    // SAFETY: BCryptGetFipsAlgorithmMode writes to a valid u8 pointer
    let status = unsafe { BCryptGetFipsAlgorithmMode(&mut fips_enabled) };
    if status.is_err() {
        return Err(Error::from_hresult(status.to_hresult()));
    }

    if fips_enabled == 0 {
        return Ok(false);
    }

    // Step 2: Open algorithm provider
    let algorithm_wide = wide_null(OsStr::new(algorithm_name));
    let mut alg_handle = BCRYPT_ALG_HANDLE::default();

    // Determine if we need HMAC flag for hash algorithms
    let flags = match algorithm_name {
        "SHA256" | "SHA384" | "SHA512" => BCRYPT_ALG_HANDLE_HMAC_FLAG,
        _ => BCRYPT_OPEN_ALGORITHM_PROVIDER_FLAGS::default(),
    };

    // SAFETY: BCryptOpenAlgorithmProvider writes to a valid BCRYPT_ALG_HANDLE pointer
    let status = unsafe {
        BCryptOpenAlgorithmProvider(
            &mut alg_handle,
            PCWSTR(algorithm_wide.as_ptr()),
            None,
            flags,
        )
    };

    if status.is_err() {
        return Err(Error::from_hresult(status.to_hresult()));
    }

    // Ensure we close the handle even if subsequent operations fail
    let result = check_fips_property(alg_handle);

    // SAFETY: We're closing a valid handle that was successfully opened
    unsafe {
        let _ = BCryptCloseAlgorithmProvider(alg_handle, 0);
    }

    result
}

fn check_fips_property(alg_handle: BCRYPT_ALG_HANDLE) -> Result<bool> {
    let property_name = wide_null(OsStr::new("FIPSAlgorithm"));
    let mut fips_approved = [0u8; 4];
    let mut result_size = 0u32;

    // SAFETY: BCryptGetProperty writes to valid buffers
    let status = unsafe {
        BCryptGetProperty(
            alg_handle.into(), // Convert BCRYPT_ALG_HANDLE to BCRYPT_HANDLE
            PCWSTR(property_name.as_ptr()),
            Some(&mut fips_approved),
            &mut result_size,
            0,
        )
    };

    if status.is_err() {
        return Err(Error::from_hresult(status.to_hresult()));
    }

    // Convert bytes to u32 (little-endian)
    let fips_approved = u32::from_le_bytes(fips_approved);
    Ok(fips_approved != 0)
}
