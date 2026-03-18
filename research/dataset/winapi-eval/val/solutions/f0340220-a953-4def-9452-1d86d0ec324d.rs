use windows::core::{Error, Result, HRESULT, PCWSTR};
use windows::Win32::Foundation::{NTSTATUS, STATUS_SUCCESS};
use windows::Win32::Security::Cryptography::{
    BCryptCloseAlgorithmProvider, BCryptCreateHash, BCryptDestroyHash, BCryptFinishHash,
    BCryptHashData, BCryptOpenAlgorithmProvider, BCRYPT_ALG_HANDLE, BCRYPT_HASH_HANDLE,
    BCRYPT_HASH_REUSABLE_FLAG, BCRYPT_SHA256_ALGORITHM,
};

fn check_ntstatus(status: NTSTATUS) -> Result<()> {
    if status != STATUS_SUCCESS {
        // Convert NTSTATUS to i32 using .0 field
        Err(Error::from_hresult(HRESULT::from_nt(status.0)))
    } else {
        Ok(())
    }
}

pub fn sha256(data: &[u8]) -> Result<[u8; 32]> {
    let mut alg_handle = BCRYPT_ALG_HANDLE::default();
    let mut hash_handle = BCRYPT_HASH_HANDLE::default();
    let mut digest = [0u8; 32];

    // Open algorithm provider
    unsafe {
        let status = BCryptOpenAlgorithmProvider(
            &mut alg_handle,
            BCRYPT_SHA256_ALGORITHM,
            PCWSTR::null(),
            BCRYPT_HASH_REUSABLE_FLAG,
        );
        check_ntstatus(status)?;
    }

    // Create hash object
    unsafe {
        let status = BCryptCreateHash(alg_handle, &mut hash_handle, None, None, 0);
        if let Err(e) = check_ntstatus(status) {
            // Clean up algorithm handle before returning error
            let _ = BCryptCloseAlgorithmProvider(alg_handle, 0);
            return Err(e);
        }
    }

    // Hash the data
    unsafe {
        let status = BCryptHashData(hash_handle, data, 0);
        if let Err(e) = check_ntstatus(status) {
            // Clean up both handles before returning error
            let _ = BCryptDestroyHash(hash_handle);
            let _ = BCryptCloseAlgorithmProvider(alg_handle, 0);
            return Err(e);
        }
    }

    // Finish hash and get digest
    unsafe {
        let status = BCryptFinishHash(hash_handle, &mut digest, 0);
        if let Err(e) = check_ntstatus(status) {
            // Clean up both handles before returning error
            let _ = BCryptDestroyHash(hash_handle);
            let _ = BCryptCloseAlgorithmProvider(alg_handle, 0);
            return Err(e);
        }
    }

    // Clean up handles
    unsafe {
        let _ = BCryptDestroyHash(hash_handle);
        let _ = BCryptCloseAlgorithmProvider(alg_handle, 0);
    }

    Ok(digest)
}
