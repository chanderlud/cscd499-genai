use std::ffi::OsStr;
use windows::core::{Error, Result, PCWSTR};
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
    let mut fips_enabled = 0u8;
    let status = unsafe { BCryptGetFipsAlgorithmMode(&mut fips_enabled) };
    if status.is_err() {
        return Err(Error::from_hresult(status.to_hresult()));
    }

    if fips_enabled == 0 {
        return Ok(false);
    }

    let algorithm_wide = wide_null(OsStr::new(algorithm_name));
    let mut alg_handle = BCRYPT_ALG_HANDLE::default();

    let flags = match algorithm_name {
        "SHA256" | "SHA384" | "SHA512" => BCRYPT_ALG_HANDLE_HMAC_FLAG,
        _ => BCRYPT_OPEN_ALGORITHM_PROVIDER_FLAGS::default(),
    };

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

    let result = check_fips_property(alg_handle);

    unsafe {
        let _ = BCryptCloseAlgorithmProvider(alg_handle, 0);
    }

    result
}

fn check_fips_property(alg_handle: BCRYPT_ALG_HANDLE) -> Result<bool> {
    let property_name = wide_null(OsStr::new("FIPSAlgorithm"));
    let mut fips_approved = [0u8; 4];
    let mut result_size = 0u32;

    let status = unsafe {
        BCryptGetProperty(
            alg_handle.into(),
            PCWSTR(property_name.as_ptr()),
            Some(&mut fips_approved),
            &mut result_size,
            0,
        )
    };

    if status.is_err() {
        return Err(Error::from_hresult(status.to_hresult()));
    }

    let fips_approved = u32::from_le_bytes(fips_approved);
    Ok(fips_approved != 0)
}

fn main() {
    // Example usage - this resolves dead code warnings
    let algorithms = ["SHA256", "AES", "MD5"];
    for algo in algorithms {
        match is_algorithm_fips_approved(algo) {
            Ok(fips) => println!("{} FIPS approved: {}", algo, fips),
            Err(e) => println!("{} error: {}", algo, e),
        }
    }
}
