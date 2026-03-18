use windows::core::{Error, Result, HRESULT};
use windows::Win32::Foundation::E_INVALIDARG;
use windows::Win32::Security::Cryptography::{BCryptDeriveKeyPBKDF2, BCRYPT_ALG_HANDLE};

pub fn pbkdf2_derive(
    alg_handle: BCRYPT_ALG_HANDLE,
    password: &[u8],
    salt: &[u8],
    iterations: u32,
    key_length: usize,
) -> Result<Vec<u8>> {
    // Validate key_length fits in u32 for BCrypt API
    if key_length > u32::MAX as usize {
        return Err(Error::from_hresult(E_INVALIDARG));
    }

    // Validate iterations is non-zero (PBKDF2 requirement)
    if iterations == 0 {
        return Err(Error::from_hresult(E_INVALIDARG));
    }

    let mut derived_key = vec![0u8; key_length];

    // SAFETY: BCryptDeriveKeyPBKDF2 is thread-safe when using the same algorithm handle.
    // The windows crate wraps the raw API to use Rust slices instead of raw pointers.
    let status = unsafe {
        BCryptDeriveKeyPBKDF2(
            alg_handle,
            Some(password),
            Some(salt),
            iterations as u64,
            &mut derived_key,
            0, // dwFlags - reserved, must be 0
        )
    };

    // Convert NTSTATUS to Result
    if status.0 < 0 {
        // Convert NTSTATUS to HRESULT, then to Error
        let hresult = HRESULT::from_win32(status.0 as u32);
        return Err(Error::from_hresult(hresult));
    }

    Ok(derived_key)
}
