use windows::core::{Error, Result, HRESULT, PCWSTR};
use windows::Win32::Foundation::{E_INVALIDARG, STATUS_SUCCESS};
use windows::Win32::Security::Cryptography::*;

fn hkdf_derive(
    algorithm: PCWSTR,
    ikm: &[u8],
    salt: Option<&[u8]>,
    info: Option<&[u8]>,
    okm_length: usize,
) -> Result<Vec<u8>> {
    // Validate output length (RFC 5869 Section 2.3)
    if okm_length == 0 || okm_length > 255 * 64 {
        // Max hash length is 64 for SHA-512
        return Err(Error::from_hresult(E_INVALIDARG));
    }

    // Open algorithm handle for HMAC
    let mut alg_handle = BCRYPT_ALG_HANDLE::default();
    let status = unsafe {
        BCryptOpenAlgorithmProvider(
            &mut alg_handle,
            algorithm,
            PCWSTR::null(),
            BCRYPT_ALG_HANDLE_HMAC_FLAG,
        )
    };
    if status != STATUS_SUCCESS {
        return Err(Error::from_hresult(HRESULT::from_win32(status.0 as u32)));
    }

    // Ensure algorithm handle is closed on exit
    let _alg_guard = AlgorithmHandleGuard(alg_handle);

    // Get hash length for the algorithm
    let mut hash_length_bytes = [0u8; 4];
    let mut result_length = 0u32;
    let status = unsafe {
        BCryptGetProperty(
            alg_handle.into(),
            BCRYPT_HASH_LENGTH,
            Some(&mut hash_length_bytes),
            &mut result_length,
            0,
        )
    };
    if status != STATUS_SUCCESS {
        return Err(Error::from_hresult(HRESULT::from_win32(status.0 as u32)));
    }
    let hash_length = u32::from_le_bytes(hash_length_bytes) as usize;

    // Validate okm_length against hash length
    if okm_length > 255 * hash_length {
        return Err(Error::from_hresult(E_INVALIDARG));
    }

    // Extract step: PRK = HMAC-Salt(IKM)
    let prk = hkdf_extract(alg_handle, ikm, salt, hash_length)?;

    // Expand step: OKM = HKDF-Expand(PRK, info, L)
    hkdf_expand(alg_handle, &prk, info, okm_length, hash_length)
}

fn hkdf_extract(
    alg_handle: BCRYPT_ALG_HANDLE,
    ikm: &[u8],
    salt: Option<&[u8]>,
    hash_length: usize,
) -> Result<Vec<u8>> {
    // Use zeroed salt if none provided (RFC 5869 Section 2.2)
    let default_salt = vec![0u8; hash_length];
    let salt = salt.unwrap_or(&default_salt);

    // Create HMAC hash with salt as key
    let mut hash_handle = BCRYPT_HASH_HANDLE::default();
    let status = unsafe { BCryptCreateHash(alg_handle, &mut hash_handle, None, Some(salt), 0) };
    if status != STATUS_SUCCESS {
        return Err(Error::from_hresult(HRESULT::from_win32(status.0 as u32)));
    }

    // Ensure hash handle is destroyed on exit
    let _hash_guard = HashHandleGuard(hash_handle);

    // Hash the IKM
    let status = unsafe { BCryptHashData(hash_handle, ikm, 0) };
    if status != STATUS_SUCCESS {
        return Err(Error::from_hresult(HRESULT::from_win32(status.0 as u32)));
    }

    // Get the hash result (PRK)
    let mut prk = vec![0u8; hash_length];
    let status = unsafe { BCryptFinishHash(hash_handle, &mut prk, 0) };
    if status != STATUS_SUCCESS {
        return Err(Error::from_hresult(HRESULT::from_win32(status.0 as u32)));
    }

    Ok(prk)
}

fn hkdf_expand(
    alg_handle: BCRYPT_ALG_HANDLE,
    prk: &[u8],
    info: Option<&[u8]>,
    okm_length: usize,
    hash_length: usize,
) -> Result<Vec<u8>> {
    let info = info.unwrap_or(&[]);
    let iterations = (okm_length + hash_length - 1) / hash_length; // ceil(okm_length / hash_length)

    let mut okm = Vec::with_capacity(okm_length);
    let mut t = Vec::new(); // T(i) from RFC 5869

    for i in 1..=iterations {
        // Create new HMAC hash with PRK as key
        let mut hash_handle = BCRYPT_HASH_HANDLE::default();
        let status = unsafe { BCryptCreateHash(alg_handle, &mut hash_handle, None, Some(prk), 0) };
        if status != STATUS_SUCCESS {
            return Err(Error::from_hresult(HRESULT::from_win32(status.0 as u32)));
        }

        // Ensure hash handle is destroyed on exit
        let _hash_guard = HashHandleGuard(hash_handle);

        // Hash T(i-1) || info || i
        if !t.is_empty() {
            let status = unsafe { BCryptHashData(hash_handle, &t, 0) };
            if status != STATUS_SUCCESS {
                return Err(Error::from_hresult(HRESULT::from_win32(status.0 as u32)));
            }
        }

        let status = unsafe { BCryptHashData(hash_handle, info, 0) };
        if status != STATUS_SUCCESS {
            return Err(Error::from_hresult(HRESULT::from_win32(status.0 as u32)));
        }

        let counter = i as u8;
        let status = unsafe { BCryptHashData(hash_handle, &[counter], 0) };
        if status != STATUS_SUCCESS {
            return Err(Error::from_hresult(HRESULT::from_win32(status.0 as u32)));
        }

        // Get T(i)
        t.resize(hash_length, 0);
        let status = unsafe { BCryptFinishHash(hash_handle, &mut t, 0) };
        if status != STATUS_SUCCESS {
            return Err(Error::from_hresult(HRESULT::from_win32(status.0 as u32)));
        }

        // Append T(i) to OKM
        okm.extend_from_slice(&t);
    }

    // Truncate to requested length
    okm.truncate(okm_length);
    Ok(okm)
}

// RAII guards for BCrypt handles
struct AlgorithmHandleGuard(BCRYPT_ALG_HANDLE);
impl Drop for AlgorithmHandleGuard {
    fn drop(&mut self) {
        unsafe {
            let _ = BCryptCloseAlgorithmProvider(self.0, 0);
        }
    }
}

struct HashHandleGuard(BCRYPT_HASH_HANDLE);
impl Drop for HashHandleGuard {
    fn drop(&mut self) {
        unsafe {
            let _ = BCryptDestroyHash(self.0);
        }
    }
}
