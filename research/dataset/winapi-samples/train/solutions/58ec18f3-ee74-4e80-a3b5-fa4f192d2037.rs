use windows::core::{Error, Result, HRESULT};
use windows::Win32::Foundation::STATUS_SUCCESS;
use windows::Win32::Security::Cryptography::{
    BCryptCloseAlgorithmProvider, BCryptCreateHash, BCryptDestroyHash, BCryptFinishHash,
    BCryptGetFipsAlgorithmMode, BCryptHashData, BCryptOpenAlgorithmProvider, BCRYPT_ALG_HANDLE,
    BCRYPT_HASH_HANDLE, BCRYPT_OPEN_ALGORITHM_PROVIDER_FLAGS, BCRYPT_SHA256_ALGORITHM,
};

// Define the FIPS flag constant since it's not exposed by the windows crate
const BCRYPT_FIPS_ALGORITHM_FLAG: u32 = 0x00000001;

fn compute_fips_sha256_streaming<'a>(
    chunks: impl IntoIterator<Item = &'a [u8]>,
) -> Result<[u8; 32]> {
    // 1. Check FIPS mode
    let mut fips_enabled: u8 = 0;
    // SAFETY: Calling FFI function with valid pointer to u8
    unsafe {
        let status = BCryptGetFipsAlgorithmMode(&mut fips_enabled);
        if status != STATUS_SUCCESS {
            return Err(Error::from_hresult(status.to_hresult()));
        }
    }
    if fips_enabled == 0 {
        return Err(Error::from_hresult(HRESULT::from_win32(0x80070000 | 1))); // FIPS_NOT_ENABLED
    }

    // 2. Open SHA-256 algorithm provider with FIPS flag
    let mut alg_handle = BCRYPT_ALG_HANDLE::default();
    // SAFETY: Calling FFI with valid pointers and null-terminated algorithm string
    unsafe {
        let status = BCryptOpenAlgorithmProvider(
            &mut alg_handle,
            BCRYPT_SHA256_ALGORITHM,
            None,
            BCRYPT_OPEN_ALGORITHM_PROVIDER_FLAGS(BCRYPT_FIPS_ALGORITHM_FLAG),
        );
        if status != STATUS_SUCCESS {
            return Err(Error::from_hresult(status.to_hresult()));
        }
    }

    // 3. Create hash object
    let mut hash_handle = BCRYPT_HASH_HANDLE::default();
    // SAFETY: Calling FFI with valid algorithm handle and pointers
    unsafe {
        let status = BCryptCreateHash(alg_handle, &mut hash_handle, None, None, 0);
        if status != STATUS_SUCCESS {
            // Clean up algorithm handle before returning error
            let _ = unsafe { BCryptCloseAlgorithmProvider(alg_handle, 0) };
            return Err(Error::from_hresult(status.to_hresult()));
        }
    }

    // 4. Hash data chunks sequentially
    for chunk in chunks {
        if chunk.is_empty() {
            continue;
        }
        // SAFETY: Calling FFI with valid hash handle and data slice
        unsafe {
            let status = BCryptHashData(hash_handle, chunk, 0);
            if status != STATUS_SUCCESS {
                // Clean up both handles before returning error
                let _ = unsafe { BCryptDestroyHash(hash_handle) };
                let _ = unsafe { BCryptCloseAlgorithmProvider(alg_handle, 0) };
                return Err(Error::from_hresult(status.to_hresult()));
            }
        }
    }

    // 5. Retrieve 32-byte digest
    let mut digest = [0u8; 32];
    // SAFETY: Calling FFI with valid hash handle and output buffer
    unsafe {
        let status = BCryptFinishHash(hash_handle, &mut digest, 0);
        if status != STATUS_SUCCESS {
            // Clean up both handles before returning error
            let _ = unsafe { BCryptDestroyHash(hash_handle) };
            let _ = unsafe { BCryptCloseAlgorithmProvider(alg_handle, 0) };
            return Err(Error::from_hresult(status.to_hresult()));
        }
    }

    // 6. Clean up handles (ignore errors during cleanup)
    // SAFETY: Handles are valid and being destroyed/closed
    unsafe {
        let _ = BCryptDestroyHash(hash_handle);
        let _ = BCryptCloseAlgorithmProvider(alg_handle, 0);
    }

    Ok(digest)
}
