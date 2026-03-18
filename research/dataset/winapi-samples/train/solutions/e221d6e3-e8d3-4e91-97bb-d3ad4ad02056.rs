use windows::core::{Error, Result, HRESULT};
use windows::Win32::Foundation::{ERROR_NOT_SUPPORTED, NTSTATUS, STATUS_SUCCESS};
use windows::Win32::Security::Cryptography::*;

const BCRYPT_FIPS_ALGORITHM_FLAG: u32 = 0x00000001;

fn compute_fips_sha256(data: &[u8]) -> Result<[u8; 32]> {
    // Verify FIPS mode is enabled
    let mut fips_enabled = 0u8;
    // SAFETY: BCryptGetFipsAlgorithmMode writes to a valid u8 pointer
    let status = unsafe { BCryptGetFipsAlgorithmMode(&mut fips_enabled) };
    if status != STATUS_SUCCESS {
        return Err(Error::from_hresult(HRESULT::from_win32(status.0 as u32)));
    }
    if fips_enabled == 0 {
        return Err(Error::from_hresult(HRESULT::from_win32(
            ERROR_NOT_SUPPORTED.0,
        )));
    }

    // Open SHA-256 algorithm provider with FIPS flag
    let mut alg_handle = BCRYPT_ALG_HANDLE::default();
    // SAFETY: BCryptOpenAlgorithmProvider writes to a valid BCRYPT_ALG_HANDLE pointer
    let status = unsafe {
        BCryptOpenAlgorithmProvider(
            &mut alg_handle,
            BCRYPT_SHA256_ALGORITHM,
            None,
            BCRYPT_OPEN_ALGORITHM_PROVIDER_FLAGS(BCRYPT_FIPS_ALGORITHM_FLAG),
        )
    };
    if status != STATUS_SUCCESS {
        return Err(Error::from_hresult(HRESULT::from_win32(status.0 as u32)));
    }

    // Create hash object
    let mut hash_handle = BCRYPT_HASH_HANDLE::default();
    // SAFETY: BCryptCreateHash writes to a valid BCRYPT_HASH_HANDLE pointer
    let status = unsafe { BCryptCreateHash(alg_handle, &mut hash_handle, None, None, 0) };
    if status != STATUS_SUCCESS {
        // Clean up algorithm handle before returning error
        unsafe { BCryptCloseAlgorithmProvider(alg_handle, 0) };
        return Err(Error::from_hresult(HRESULT::from_win32(status.0 as u32)));
    }

    // Hash the input data
    // SAFETY: BCryptHashData reads from a valid byte slice
    let status = unsafe { BCryptHashData(hash_handle, data, 0) };
    if status != STATUS_SUCCESS {
        // Clean up both handles before returning error
        unsafe {
            BCryptDestroyHash(hash_handle);
            BCryptCloseAlgorithmProvider(alg_handle, 0);
        };
        return Err(Error::from_hresult(HRESULT::from_win32(status.0 as u32)));
    }

    // Get the hash value
    let mut hash = [0u8; 32];
    // SAFETY: BCryptFinishHash writes to a valid byte array of correct size
    let status = unsafe { BCryptFinishHash(hash_handle, &mut hash, 0) };

    // Clean up handles regardless of success/failure
    unsafe {
        BCryptDestroyHash(hash_handle);
        BCryptCloseAlgorithmProvider(alg_handle, 0);
    };

    if status != STATUS_SUCCESS {
        return Err(Error::from_hresult(HRESULT::from_win32(status.0 as u32)));
    }

    Ok(hash)
}
