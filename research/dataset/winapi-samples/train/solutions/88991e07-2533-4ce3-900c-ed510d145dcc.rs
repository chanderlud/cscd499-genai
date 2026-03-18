use std::mem::MaybeUninit;
use windows::core::{Error, Result, HRESULT};
use windows::Win32::Security::Cryptography::{
    BCryptCloseAlgorithmProvider, BCryptDeriveKeyPBKDF2, BCryptOpenAlgorithmProvider,
    BCRYPT_ALG_HANDLE, BCRYPT_OPEN_ALGORITHM_PROVIDER_FLAGS, BCRYPT_SHA256_ALGORITHM,
};

fn derive_key_pbkdf2(
    password: &[u8],
    salt: &[u8],
    iterations: u32,
    key_length: usize,
) -> Result<Vec<u8>> {
    // Validate inputs
    if password.is_empty() {
        return Err(Error::from_hresult(HRESULT::from_win32(0x80070057))); // E_INVALIDARG
    }
    if iterations == 0 {
        return Err(Error::from_hresult(HRESULT::from_win32(0x80070057))); // E_INVALIDARG
    }
    if key_length == 0 {
        return Err(Error::from_hresult(HRESULT::from_win32(0x80070057))); // E_INVALIDARG
    }

    // Open algorithm provider for SHA256
    let mut alg_handle = MaybeUninit::<BCRYPT_ALG_HANDLE>::uninit();

    // SAFETY: BCryptOpenAlgorithmProvider is a valid Windows API call
    let nt_status = unsafe {
        BCryptOpenAlgorithmProvider(
            alg_handle.as_mut_ptr(),
            BCRYPT_SHA256_ALGORITHM,
            None,
            BCRYPT_OPEN_ALGORITHM_PROVIDER_FLAGS::default(),
        )
    };

    if nt_status.0 != 0 {
        return Err(Error::from_hresult(HRESULT::from_win32(nt_status.0 as u32)));
    }

    // SAFETY: alg_handle is now initialized
    let alg_handle = unsafe { alg_handle.assume_init() };

    // Ensure we close the algorithm handle even if derivation fails
    let result = derive_key_inner(alg_handle, password, salt, iterations, key_length);

    // SAFETY: BCryptCloseAlgorithmProvider is a valid Windows API call
    unsafe {
        BCryptCloseAlgorithmProvider(alg_handle, 0);
    }

    result
}

fn derive_key_inner(
    alg_handle: BCRYPT_ALG_HANDLE,
    password: &[u8],
    salt: &[u8],
    iterations: u32,
    key_length: usize,
) -> Result<Vec<u8>> {
    let mut derived_key = vec![0u8; key_length];

    // SAFETY: BCryptDeriveKeyPBKDF2 is a valid Windows API call
    // All slices are valid for the duration of the call
    let nt_status = unsafe {
        BCryptDeriveKeyPBKDF2(
            alg_handle,
            Some(password),
            Some(salt),
            iterations as u64,
            &mut derived_key,
            0,
        )
    };

    if nt_status.0 != 0 {
        return Err(Error::from_hresult(HRESULT::from_win32(nt_status.0 as u32)));
    }

    Ok(derived_key)
}
