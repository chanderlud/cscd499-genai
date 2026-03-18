use std::cell::RefCell;
use windows::core::{Result, PCWSTR};
use windows::Win32::Foundation::NTSTATUS;
use windows::Win32::Security::Cryptography::{
    BCryptCloseAlgorithmProvider, BCryptCreateHash, BCryptDestroyHash, BCryptFinishHash,
    BCryptHashData, BCryptOpenAlgorithmProvider, BCRYPT_ALG_HANDLE, BCRYPT_HASH_HANDLE,
    BCRYPT_OPEN_ALGORITHM_PROVIDER_FLAGS, BCRYPT_SHA256_ALGORITHM,
};

thread_local! {
    static SHA256_ALG_HANDLE: RefCell<Option<BCRYPT_ALG_HANDLE>> = RefCell::new(None);
}

fn get_sha256_algorithm() -> Result<BCRYPT_ALG_HANDLE> {
    SHA256_ALG_HANDLE.with(|cell| {
        let mut handle_ref = cell.borrow_mut();
        if let Some(handle) = *handle_ref {
            return Ok(handle);
        }

        let mut handle = BCRYPT_ALG_HANDLE::default();
        // SAFETY: FFI call with valid parameters
        let status = unsafe {
            BCryptOpenAlgorithmProvider(
                &mut handle,
                BCRYPT_SHA256_ALGORITHM,
                PCWSTR::null(),
                BCRYPT_OPEN_ALGORITHM_PROVIDER_FLAGS(0),
            )
        };
        status.ok()?;

        *handle_ref = Some(handle);
        Ok(handle)
    })
}

fn check_status(status: NTSTATUS) -> Result<()> {
    status.ok()
}

pub fn sha256_hash(data: &[u8]) -> Result<[u8; 32]> {
    let algorithm_handle = get_sha256_algorithm()?;

    let mut hash_handle = BCRYPT_HASH_HANDLE::default();
    // SAFETY: FFI call with valid parameters
    let status = unsafe { BCryptCreateHash(algorithm_handle, &mut hash_handle, None, None, 0) };
    check_status(status)?;

    // Ensure hash handle is destroyed even if subsequent operations fail
    let result = (|| {
        // SAFETY: FFI call with valid parameters and data slice
        let status = unsafe { BCryptHashData(hash_handle, data, 0) };
        check_status(status)?;

        let mut hash = [0u8; 32];
        // SAFETY: FFI call with valid parameters and output buffer slice
        let status = unsafe { BCryptFinishHash(hash_handle, &mut hash, 0) };
        check_status(status)?;

        Ok(hash)
    })();

    // SAFETY: FFI call to destroy the hash handle
    unsafe { BCryptDestroyHash(hash_handle) };
    result
}
