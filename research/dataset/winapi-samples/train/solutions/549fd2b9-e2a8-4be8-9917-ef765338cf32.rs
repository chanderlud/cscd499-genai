use windows::core::{Error, Result, HRESULT, PCWSTR};
use windows::Win32::Security::Cryptography::{
    BCryptCloseAlgorithmProvider, BCryptCreateHash, BCryptDestroyHash, BCryptFinishHash,
    BCryptHashData, BCryptOpenAlgorithmProvider, BCRYPT_ALG_HANDLE, BCRYPT_ALG_HANDLE_HMAC_FLAG,
    BCRYPT_HASH_HANDLE, BCRYPT_SHA256_ALGORITHM, BCRYPT_SHA384_ALGORITHM, BCRYPT_SHA512_ALGORITHM,
};

#[derive(Debug, Clone, Copy)]
pub enum HmacAlgorithm {
    Sha256,
    Sha384,
    Sha512,
}

fn get_algorithm_id(algorithm: HmacAlgorithm) -> PCWSTR {
    match algorithm {
        HmacAlgorithm::Sha256 => BCRYPT_SHA256_ALGORITHM,
        HmacAlgorithm::Sha384 => BCRYPT_SHA384_ALGORITHM,
        HmacAlgorithm::Sha512 => BCRYPT_SHA512_ALGORITHM,
    }
}

fn get_hash_length(algorithm: HmacAlgorithm) -> u32 {
    match algorithm {
        HmacAlgorithm::Sha256 => 32,
        HmacAlgorithm::Sha384 => 48,
        HmacAlgorithm::Sha512 => 64,
    }
}

pub fn compute_hmac(algorithm: HmacAlgorithm, key: &[u8], message: &[u8]) -> Result<Vec<u8>> {
    let alg_id = get_algorithm_id(algorithm);

    // Open algorithm provider with HMAC flag
    let mut alg_handle = BCRYPT_ALG_HANDLE::default();
    // SAFETY: BCryptOpenAlgorithmProvider is safe to call with valid parameters
    let status = unsafe {
        BCryptOpenAlgorithmProvider(
            &mut alg_handle,
            alg_id,
            PCWSTR::null(),
            BCRYPT_ALG_HANDLE_HMAC_FLAG,
        )
    };
    if status.0 < 0 {
        return Err(Error::from_hresult(HRESULT::from_nt(status.0)));
    }

    // Create hash with key
    let mut hash_handle = BCRYPT_HASH_HANDLE::default();
    // SAFETY: BCryptCreateHash is safe to call with valid parameters
    let status = unsafe { BCryptCreateHash(alg_handle, &mut hash_handle, None, Some(key), 0) };
    if status.0 < 0 {
        // Clean up algorithm handle before returning error
        unsafe { BCryptCloseAlgorithmProvider(alg_handle, 0) };
        return Err(Error::from_hresult(HRESULT::from_nt(status.0)));
    }

    // Hash the message data
    // SAFETY: BCryptHashData is safe to call with valid parameters
    let status = unsafe { BCryptHashData(hash_handle, message, 0) };
    if status.0 < 0 {
        // Clean up handles before returning error
        unsafe {
            BCryptDestroyHash(hash_handle);
            BCryptCloseAlgorithmProvider(alg_handle, 0);
        };
        return Err(Error::from_hresult(HRESULT::from_nt(status.0)));
    }

    // Get hash length and allocate buffer
    let hash_len = get_hash_length(algorithm);
    let mut digest = vec![0u8; hash_len as usize];

    // Finish hash computation
    // SAFETY: BCryptFinishHash is safe to call with valid parameters
    let status = unsafe { BCryptFinishHash(hash_handle, &mut digest, 0) };

    // Clean up handles regardless of success/failure
    unsafe {
        BCryptDestroyHash(hash_handle);
        BCryptCloseAlgorithmProvider(alg_handle, 0);
    };

    if status.0 < 0 {
        return Err(Error::from_hresult(HRESULT::from_nt(status.0)));
    }

    Ok(digest)
}
