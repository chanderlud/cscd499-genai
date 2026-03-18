use windows::core::{Error, Result, HRESULT, PCWSTR};
use windows::Win32::Security::Cryptography::{
    BCryptCloseAlgorithmProvider, BCryptCreateHash, BCryptDestroyHash, BCryptFinishHash,
    BCryptGetProperty, BCryptHashData, BCryptOpenAlgorithmProvider, BCRYPT_ALGORITHM_NAME,
    BCRYPT_ALG_HANDLE, BCRYPT_ALG_HANDLE_HMAC_FLAG, BCRYPT_HASH_HANDLE, BCRYPT_HASH_LENGTH,
};

struct HmacAlgorithmHandle(BCRYPT_ALG_HANDLE);

impl Drop for HmacAlgorithmHandle {
    fn drop(&mut self) {
        unsafe {
            let _ = BCryptCloseAlgorithmProvider(self.0, 0);
        }
    }
}

struct HashHandle(BCRYPT_HASH_HANDLE);

impl Drop for HashHandle {
    fn drop(&mut self) {
        unsafe {
            let _ = BCryptDestroyHash(self.0);
        }
    }
}

fn get_hash_length(alg_handle: BCRYPT_ALG_HANDLE) -> Result<u32> {
    let mut hash_len = 0u32;
    let mut result_len = 0u32;

    unsafe {
        let status = BCryptGetProperty(
            alg_handle.into(),
            BCRYPT_HASH_LENGTH,
            Some(std::slice::from_raw_parts_mut(
                &mut hash_len as *mut u32 as *mut u8,
                std::mem::size_of::<u32>(),
            )),
            &mut result_len as *mut u32,
            0,
        );

        if status.0 != 0 {
            return Err(Error::from_hresult(HRESULT::from_win32(status.0 as u32)));
        }
    }

    Ok(hash_len)
}

fn open_hmac_algorithm(alg_handle: BCRYPT_ALG_HANDLE) -> Result<HmacAlgorithmHandle> {
    let mut name_len = 0u32;
    let mut result_len = 0u32;

    // Get algorithm name length
    unsafe {
        let status = BCryptGetProperty(
            alg_handle.into(),
            BCRYPT_ALGORITHM_NAME,
            None,
            &mut result_len as *mut u32,
            0,
        );

        if status.0 != 0 {
            return Err(Error::from_hresult(HRESULT::from_win32(status.0 as u32)));
        }
    }

    let mut name_buffer = vec![0u16; result_len as usize / 2];

    // Get algorithm name
    unsafe {
        let status = BCryptGetProperty(
            alg_handle.into(),
            BCRYPT_ALGORITHM_NAME,
            Some(std::slice::from_raw_parts_mut(
                name_buffer.as_mut_ptr() as *mut u8,
                name_buffer.len() * 2,
            )),
            &mut name_len as *mut u32,
            0,
        );

        if status.0 != 0 {
            return Err(Error::from_hresult(HRESULT::from_win32(status.0 as u32)));
        }
    }

    // Open HMAC algorithm
    let mut hmac_alg_handle = BCRYPT_ALG_HANDLE::default();
    unsafe {
        let status = BCryptOpenAlgorithmProvider(
            &mut hmac_alg_handle,
            PCWSTR(name_buffer.as_ptr()),
            PCWSTR::null(),
            BCRYPT_ALG_HANDLE_HMAC_FLAG,
        );

        if status.0 != 0 {
            return Err(Error::from_hresult(HRESULT::from_win32(status.0 as u32)));
        }
    }

    Ok(HmacAlgorithmHandle(hmac_alg_handle))
}

fn hmac_hash(hmac_alg_handle: BCRYPT_ALG_HANDLE, key: &[u8], data: &[u8]) -> Result<Vec<u8>> {
    let mut hash_handle = BCRYPT_HASH_HANDLE::default();

    // Create HMAC hash with key
    unsafe {
        let status = BCryptCreateHash(hmac_alg_handle, &mut hash_handle, None, Some(key), 0);

        if status.0 != 0 {
            return Err(Error::from_hresult(HRESULT::from_win32(status.0 as u32)));
        }
    }

    let hash_handle = HashHandle(hash_handle);

    // Hash the data
    if !data.is_empty() {
        unsafe {
            let status = BCryptHashData(hash_handle.0, data, 0);

            if status.0 != 0 {
                return Err(Error::from_hresult(HRESULT::from_win32(status.0 as u32)));
            }
        }
    }

    // Get hash length
    let hash_len = get_hash_length(hmac_alg_handle)?;
    let mut hash = vec![0u8; hash_len as usize];

    // Finish hash
    unsafe {
        let status = BCryptFinishHash(hash_handle.0, &mut hash, 0);

        if status.0 != 0 {
            return Err(Error::from_hresult(HRESULT::from_win32(status.0 as u32)));
        }
    }

    Ok(hash)
}

fn hkdf_derive(
    alg_handle: BCRYPT_ALG_HANDLE,
    ikm: &[u8],
    salt: Option<&[u8]>,
    info: Option<&[u8]>,
    output_len: usize,
) -> Result<Vec<u8>> {
    let hash_len = get_hash_length(alg_handle)? as usize;

    // Open HMAC algorithm
    let hmac_alg_handle = open_hmac_algorithm(alg_handle)?;

    // Extract phase: PRK = HMAC-Hash(salt, IKM)
    let actual_salt = match salt {
        Some(s) => s.to_vec(),
        None => vec![0u8; hash_len],
    };

    let prk = hmac_hash(hmac_alg_handle.0, &actual_salt, ikm)?;

    // Expand phase: OKM = T(1) | T(2) | ... | T(N)
    let mut okm = Vec::with_capacity(output_len);
    let mut t = Vec::new();
    let info_bytes = info.unwrap_or(&[]);

    let n = (output_len + hash_len - 1) / hash_len;

    for i in 1..=n {
        let mut data = t.clone();
        data.extend_from_slice(info_bytes);
        data.push(i as u8);

        t = hmac_hash(hmac_alg_handle.0, &prk, &data)?;
        okm.extend_from_slice(&t);
    }

    okm.truncate(output_len);
    Ok(okm)
}
