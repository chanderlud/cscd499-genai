use windows::core::{Error, Result};
use windows::Win32::Foundation::E_INVALIDARG;
use windows::Win32::Security::Cryptography::*;

pub struct HashHandle {
    handle: BCRYPT_HASH_HANDLE,
}

impl HashHandle {
    pub fn new() -> Result<Self> {
        Ok(Self {
            handle: BCRYPT_HASH_HANDLE::default(),
        })
    }
}

impl Drop for HashHandle {
    fn drop(&mut self) {
        unsafe {
            let _ = BCryptDestroyHash(self.handle);
        }
    }
}

pub struct HkdfSha256 {
    algorithm: BCRYPT_ALG_HANDLE,
}

impl HkdfSha256 {
    pub fn new() -> Result<Self> {
        let mut algorithm = BCRYPT_ALG_HANDLE::default();

        unsafe {
            BCryptOpenAlgorithmProvider(
                &mut algorithm,
                BCRYPT_SHA256_ALGORITHM,
                None,
                BCRYPT_ALG_HANDLE_HMAC_FLAG,
            )
            .ok()?;
        }

        Ok(Self { algorithm })
    }

    pub fn extract(&self, salt: &[u8], ikm: &[u8]) -> Result<Vec<u8>> {
        let effective_salt = if salt.is_empty() {
            vec![0u8; 32]
        } else {
            salt.to_vec()
        };

        self.hmac_sha256(&effective_salt, ikm)
    }

    pub fn expand(&self, prk: &[u8], info: &[u8], length: usize) -> Result<Vec<u8>> {
        const HASH_LEN: usize = 32;

        if length > 255 * HASH_LEN {
            return Err(Error::from_hresult(E_INVALIDARG));
        }

        let mut output = Vec::with_capacity(length);
        let mut t = Vec::new();
        let mut counter: u8 = 1;

        while output.len() < length {
            let mut data = Vec::new();
            data.extend_from_slice(&t);
            data.extend_from_slice(info);
            data.push(counter);

            t = self.hmac_sha256(prk, &data)?;
            output.extend_from_slice(&t);

            counter = counter.wrapping_add(1);
            if counter == 0 {
                return Err(Error::from_hresult(E_INVALIDARG));
            }
        }

        output.truncate(length);
        Ok(output)
    }

    pub fn hmac_sha256(&self, key: &[u8], data: &[u8]) -> Result<Vec<u8>> {
        let mut hash_handle = HashHandle::new()?;

        unsafe {
            BCryptCreateHash(
                self.algorithm,
                &mut hash_handle.handle,
                None,
                if key.is_empty() { None } else { Some(key) },
                0,
            )
            .ok()?;

            if !data.is_empty() {
                BCryptHashData(hash_handle.handle, data, 0).ok()?;
            }

            let mut hash_length_bytes = [0u8; 4];
            let mut result_length = 0u32;
            BCryptGetProperty(
                self.algorithm.into(),
                BCRYPT_HASH_LENGTH,
                Some(&mut hash_length_bytes),
                &mut result_length,
                0,
            )
            .ok()?;

            let hash_length = u32::from_le_bytes(hash_length_bytes);

            let mut hash_result = vec![0u8; hash_length as usize];
            BCryptFinishHash(hash_handle.handle, &mut hash_result, 0).ok()?;

            Ok(hash_result)
        }
    }
}

impl Drop for HkdfSha256 {
    fn drop(&mut self) {
        unsafe {
            let _ = BCryptCloseAlgorithmProvider(self.algorithm, 0);
        }
    }
}

pub fn hkdf_sha256_extract_and_expand(
    salt: &[u8],
    ikm: &[u8],
    info: &[u8],
    length: usize,
) -> Result<Vec<u8>> {
    let hkdf = HkdfSha256::new()?;
    let prk = hkdf.extract(salt, ikm)?;
    hkdf.expand(&prk, info, length)
}
